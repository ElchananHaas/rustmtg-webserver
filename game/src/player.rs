use crate::actions::Action;
use crate::client_message::{Ask, AskPair, AskSelectN, ClientMessage, GameState};
use crate::game::Cards;
use anyhow::Result;
use common::counters::Counter;
use common::entities::{CardId, ManaId, PlayerId, TargetId};
use common::hashset_obj::HashSetObj;
use futures::{SinkExt, StreamExt};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::filters::ws::Message;

use warp::ws::WebSocket;
#[derive(Clone, JsonSchema, Debug)]
pub struct Player {
    pub name: String,
    pub life: i64,
    pub library: Vec<CardId>,
    pub hand: HashSetObj<CardId>,
    pub mana_pool: HashSetObj<ManaId>,
    pub graveyard: Vec<CardId>,
    pub counters: Vec<Counter>,
    pub max_handsize: usize,
    #[serde(skip)]
    pub player_con: PlayerCon,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PlayerView {
    pub name: String,
    pub life: i64,
    pub library: Vec<CardId>,
    pub hand: Vec<CardId>,
    pub graveyard: Vec<CardId>,
    pub mana_pool: HashSetObj<ManaId>,
    pub counters: Vec<Counter>,
    pub max_handsize: usize,
}
fn view_t<'a>(
    cards: &'a Cards,
    r: impl Iterator<Item = &'a CardId> + 'a,
    pl: PlayerId,
    hidden_map: &'a HashMap<CardId, CardId>,
) -> impl Iterator<Item = CardId> + 'a {
    r.map(move |&id| {
        cards
            .get(id)
            .map(|ent| {
                if ent.known_to.contains(&pl) {
                    Some(id)
                } else {
                    hidden_map.get(&id).map(|x| *x) //If the player's hand contains a card not in the game
                                                    //don't send it to the client because it will confuse the javascript
                }
            })
            .flatten()
    })
    .filter_map(|x| x)
}
impl Player {
    pub fn view(
        &self,
        cards: &Cards,
        player: PlayerId,
        hidden_map: &HashMap<CardId, CardId>,
    ) -> PlayerView {
        let libview = view_t(cards, self.library.iter(), player, hidden_map).collect::<Vec<_>>();
        let handview = view_t(cards, self.hand.iter(), player, hidden_map).collect::<Vec<_>>();
        PlayerView {
            name: self.name.clone(),
            life: self.life,
            library: libview,
            hand: handview,
            graveyard: self.graveyard.clone(),
            mana_pool: self.mana_pool.clone(),
            max_handsize: self.max_handsize,
            counters: self.counters.clone(),
        }
    }

    pub async fn send_data(&self, data: ClientMessage) -> Result<()> {
        let mut buffer = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut json_serial = serde_json::Serializer::new(cursor);
            data.serialize(&mut json_serial)?;
        }
        self.player_con.send_data(buffer).await
    }

    //Select n entities from a vector, returns selected indicies
    pub async fn ask_user_selectn<T>(&self, query: &Ask, ask: &AskSelectN<T>) -> HashSetObj<usize> {
        loop {
            self.send_data(ClientMessage::AskUser(query.clone()))
                .await
                .expect("Failed to send data");
            let response = self.player_con.receive::<HashSetObj<usize>>().await;
            let response = if let Ok(resp) = response {
                resp
            } else {
                continue;
            };
            if response.len() < ask.min.try_into().unwrap()
                || response.len() > ask.max.try_into().unwrap()
                || response.iter().any(|&i| i >= ask.ents.len())
            {
                continue;
            }
            println!("accepted {:?}", response);
            return response;
        }
    }
    //pair attackers with blockers/attacking targets
    //Returns an adjacency list with either the
    //planeswalker/player each attacker is attacking,
    //or the list of creatures each blocker is blocking
    pub async fn ask_user_pair<T: DeserializeOwned + Hash + Eq + Copy + Clone + Debug>(
        &self,
        query: &Ask,
        ask: &AskPair<T>,
    ) -> HashMap<CardId, HashSetObj<T>> {
        'outer: loop {
            self.send_data(ClientMessage::AskUser(query.clone()))
                .await
                .expect("Failed to send data");
            let response = self
                .player_con
                .receive::<HashMap<CardId, HashSetObj<T>>>()
                .await;
            let response = if let Ok(resp) = response {
                resp
            } else {
                continue 'outer;
            };
            let response: HashMap<CardId, HashSetObj<T>> = response
                .into_iter()
                .map(|(key, value)| (key, value.into_iter().collect()))
                .collect();
            for (card, pairing) in response.iter() {
                if let Some(input) = ask.pairs.get(card) {
                    let items = input.items.clone();
                    if pairing.len() < input.min
                        || pairing.len() > input.max
                        || !items.is_subset(pairing)
                    {
                        continue 'outer;
                    }
                } else {
                    continue 'outer;
                }
            }
            println!("accepted {:?}", response);
            return response;
        }
    }
}

#[allow(dead_code)]
pub enum Socket {
    TestSocket(TestClient), //used in testing, and therefore isn't dead code
    Web(WebSocket),
}
#[derive(Clone)]
pub struct PlayerCon {
    socket: Arc<Mutex<Socket>>,
}
impl Debug for PlayerCon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerCon")
            .field("socket", &"socket")
            .finish()
    }
}
impl PlayerCon {
    pub fn new(socket: WebSocket) -> Self {
        PlayerCon {
            socket: Arc::new(Mutex::new(Socket::Web(socket))),
        }
    }
    #[allow(dead_code)] //Used in test code and therefore isn't dead
    pub fn new_test(test: TestClient) -> Self {
        PlayerCon {
            socket: Arc::new(Mutex::new(Socket::TestSocket(test))),
        }
    }
    pub async fn receive<T: DeserializeOwned>(&self) -> Result<T> {
        let mut socket = self.socket.lock().await;
        loop {
            let recieved = match socket.deref_mut() {
                Socket::TestSocket(test) => test.recieve(),
                Socket::Web(socket) => socket.next().await.expect("Socket is still open"),
            };
            let message = if let Ok(msg) = recieved {
                msg
            } else {
                PlayerCon::socket_error().await;
                continue;
            };
            let text = message
                .to_str()
                .map_err(|_| anyhow::Error::msg("Didn't recieve a string"))?;
            println!("parsing:{}", text);
            return serde_json::from_str(text)
                .map_err(|_| anyhow::Error::msg("Message failed to parse correctly"));
        }
    }
    pub async fn send_data(&self, data: Vec<u8>) -> Result<()> {
        let msg = std::str::from_utf8(&data).expect("json is valid text");
        let mut socket = self.socket.lock().await;
        match socket.deref_mut() {
            Socket::TestSocket(test) => test.send_message(Message::text(msg)),
            Socket::Web(socket) => socket
                .send(Message::text(msg))
                .await
                .map_err(|_| anyhow::Error::msg("Connection broke on send")),
        }
    }
    async fn socket_error() {
        panic!("Connection broke on read");
    }
}

pub struct TestClient {
    game: Option<GameState>,
    prepared_response: ClientResponse,
    mock_client: Box<dyn MockClient>,
}
impl TestClient {
    pub fn with_client(client: Box<dyn MockClient>) -> Self {
        Self {
            game: None,
            prepared_response: ClientResponse::None,
            mock_client: client,
        }
    }
    pub fn send_message(&mut self, msg: Message) -> Result<()> {
        let text = msg.to_str().expect("message is text");
        let contents: Result<ClientMessage, _> = serde_json::from_str(text);
        let contents = contents.expect("parsed correctly");
        match contents {
            ClientMessage::GameState(state) => {
                self.game = Some(state);
            }
            ClientMessage::AskUser(ask) => {
                self.prepared_response = TestClient::respond(
                    &mut self.mock_client,
                    &self.game.as_ref().expect("game is set"),
                    ask,
                );
            }
        }
        Ok(())
    }
    fn respond(
        mock_client: &mut Box<dyn MockClient>,
        game: &GameState,
        ask: Ask,
    ) -> ClientResponse {
        match &ask {
            Ask::Action(act) => {
                if act.min == 0 && act.max == 1 {
                    if act.ents.len() == 0 {
                        return ClientResponse::Indicies(HashSetObj::new());
                    }
                }
            }
            Ask::Target(ask) => {
                let resp = mock_client.select_targets(game, ask);
                return ClientResponse::Indicies(resp);
            }
            _ => (),
        };
        ClientResponse::None
    }
    pub fn recieve(&mut self) -> Result<Message, warp::Error> {
        let mut buffer = Vec::new();
        let cursor = std::io::Cursor::new(&mut buffer);
        let mut json_serial = serde_json::Serializer::new(cursor);
        match &self.prepared_response {
            ClientResponse::None => {
                panic!("response not set");
            }
            ClientResponse::Indicies(indexs) => {
                indexs
                    .serialize(&mut json_serial)
                    .expect("serialized to json correctly");
            }
        };
        let msg = std::str::from_utf8(&buffer).expect("json is valid text");
        return Ok(Message::text(msg));
    }
}
impl Default for TestClient {
    fn default() -> Self {
        Self {
            game: None,
            prepared_response: ClientResponse::None,
            mock_client: Box::new(PanicClient {}),
        }
    }
}
pub enum ClientResponse {
    None,
    Indicies(HashSetObj<usize>),
}

pub trait MockClient: Send + Sync {
    fn select_action(&mut self, _game: &GameState, ask: &AskSelectN<Action>) -> HashSetObj<usize> {
        println!("query");
        dbg!(ask);
        panic!("Select action not overriden");
    }
    fn select_targets(
        &mut self,
        _game: &GameState,
        ask: &AskSelectN<TargetId>,
    ) -> HashSetObj<usize> {
        println!("query");
        dbg!(ask);
        panic!("Select targets not overriden");
    }
}
pub struct PanicClient {}
impl MockClient for PanicClient {}
