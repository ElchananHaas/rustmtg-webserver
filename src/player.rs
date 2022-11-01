use crate::client_message::{Ask, AskPairAB, AskSelectN, ClientMessage};
use crate::entities::{CardId, ManaId, PlayerId};
use crate::game::Cards;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::Mutex;

use warp::filters::ws::Message;
use warp::ws::WebSocket;
#[derive(Clone, JsonSchema)]
pub struct Player {
    pub name: String,
    pub life: i64,
    pub library: Vec<CardId>,
    pub hand: HashSet<CardId>,
    pub mana_pool: HashSet<ManaId>,
    pub graveyard: Vec<CardId>,
    pub max_handsize: usize,
    #[serde(skip)]
    pub player_con: PlayerCon,
}

#[derive(Serialize, JsonSchema)]
pub struct PlayerView<'a> {
    pub name: &'a str,
    pub life: i64,
    pub library: Vec<CardId>,
    pub hand: Vec<CardId>,
    pub graveyard: &'a Vec<CardId>,
    pub mana_pool: &'a HashSet<ManaId>,
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
            name: &self.name,
            life: self.life,
            library: libview,
            hand: handview,
            graveyard: &self.graveyard,
            mana_pool: &self.mana_pool,
            max_handsize: self.max_handsize,
        }
    }

    pub async fn send_data(&self, data: ClientMessage<'_, '_, '_>) -> Result<()> {
        let mut buffer = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut json_serial = serde_json::Serializer::new(cursor);
            data.serialize(&mut json_serial)?;
        }
        self.player_con.send_data(buffer).await
    }

    //Select n entities from a vector, returns selected indicies
    pub async fn ask_user_selectn<T>(&self, query: &Ask, ask: &AskSelectN<T>) -> Vec<usize> {
        loop {
            self.send_data(ClientMessage::AskUser(query))
                .await
                .expect("Failed to send data");
            let response = self.player_con.receive::<Vec<usize>>().await;
            let response = if let Ok(resp) = response {
                resp
            } else {
                continue;
            };
            let response_unique: HashSet<usize> = response.iter().cloned().collect();
            if response.len() < ask.min.try_into().unwrap()
                || response.len() > ask.max.try_into().unwrap()
                || response.len() != response_unique.len()
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
        ask: &AskPairAB<T>,
    ) -> HashMap<CardId, Vec<T>> {
        'outer: loop {
            self.send_data(ClientMessage::AskUser(query))
                .await
                .expect("Failed to send data");
            let response = self.player_con.receive::<HashMap<CardId, Vec<T>>>().await;
            let response = if let Ok(resp) = response {
                resp
            } else {
                continue 'outer;
            };
            for (card, pairing) in response.iter() {
                let bounds = if let Some(bound) = ask.a.get(card) {
                    bound
                } else {
                    continue 'outer;
                };
                if pairing.len() < bounds.0 || pairing.len() > bounds.1 {
                    continue 'outer;
                }
                if pairing.len() != pairing.iter().map(|x| *x).collect::<HashSet<T>>().len() {
                    continue 'outer;
                }
            }
            println!("accepted {:?}", response);
            return response;
        }
    }
}


pub enum Socket{
    TestSocket,
    Web(WebSocket)
}
#[derive(Clone)]
pub struct PlayerCon {
    socket: Arc<Mutex<Socket>>,
}

impl PlayerCon {
    pub fn new(socket: WebSocket) -> Self {
        PlayerCon {
            socket: Arc::new(Mutex::new(Socket::Web(socket))),
        }
    }
    pub fn new_test() -> Self {
        PlayerCon { socket: Arc::new(Mutex::new(Socket::TestSocket)) }
    }
    pub async fn receive<T: DeserializeOwned>(&self) -> Result<T> {
        let mut socket = self.socket.lock().await;
        let socket: &mut WebSocket=match socket.deref_mut(){
            Socket::TestSocket=>{
                return Err(anyhow::Error::msg("Recieving Messages aren't supported in test mode yet"));
            }
            Socket::Web(sock)=> {sock}
        };
        loop {
            let recieved = socket.next().await.expect("Socket is still open");
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
    pub async fn send_data(&self, state: Vec<u8>) -> Result<()> {
        let mut socket = self.socket.lock().await;
        let socket: &mut WebSocket=match socket.deref_mut(){
            Socket::TestSocket=>{
                return Err(anyhow::Error::msg("Sending Messages aren't supported in test mode yet"));
            }
            Socket::Web(sock)=> {sock}
        };
        let msg = std::str::from_utf8(&state).expect("json is valid text");
        socket
            .send(Message::text(msg))
            .await
            .map_err(|_| anyhow::Error::msg("Connection broke on send"))
    }
    async fn socket_error() {
        panic!("Connection broke on read");
    }
}
