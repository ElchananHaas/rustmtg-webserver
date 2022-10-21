use crate::client_message::{Ask, AskPairAB, AskSelectN, ClientMessage};
use crate::entities::{CardId, ManaId, PlayerId};
use crate::game::Cards;
use anyhow::{bail, Result};
use futures::{SinkExt, StreamExt};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::hyper::Response;

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
            self.send_data(ClientMessage::AskUser(query)).await.expect("Failed to send data");
            let response = self.player_con.receive::<Vec<usize>>().await;
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
    pub async fn ask_user_pair<T: DeserializeOwned + Hash + Eq + Copy + Clone>(
        &self,
        query: &Ask,
        ask: &AskPairAB<T>,
    ) -> Vec<Vec<T>> {
        'outer: loop {
            self.send_data(ClientMessage::AskUser(query)).await.expect("Failed to send data");
            let response = self.player_con.receive::<Vec<Vec<T>>>().await;
            if response.len() != ask.a.len() {
                continue 'outer;
            }
            for (i, row) in response.iter().enumerate() {
                if row.len() < ask.num_choices[i].0 || row.len() > ask.num_choices[i].1 {
                    continue 'outer;
                }
                let as_set = row.iter().map(|x| *x).collect::<HashSet<T>>();
                if row.len() != as_set.len() {
                    continue 'outer;
                }
            }
            return response;
        }
    }
}

#[derive(Clone)]
pub struct PlayerCon {
    socket: Arc<Mutex<WebSocket>>,
}

impl PlayerCon {
    pub fn new(socket: WebSocket) -> Self {
        PlayerCon {
            socket: Arc::new(Mutex::new(socket)),
        }
    }

    pub async fn receive<T: DeserializeOwned>(&self) -> T {
        let mut socket = self.socket.lock().await;
        loop {
            let recieved = socket.next().await.expect("Socket is still open");
            let message = if let Ok(msg) = recieved {
                println!("Recieved message {:?}",msg);
                msg
            } else {
                PlayerCon::socket_error().await;
                continue;
            };
            let text = if let Ok(txt) = message.to_str() {
                txt
            } else {
                continue;
            };
            println!("parsing:{}", text);
            if let Ok(parsed) = serde_json::from_str(text) {
                println!("parsed!");
                return parsed;
            } else {
                continue;
            }
        }
    }
    pub async fn send_data(&self, state: Vec<u8>) -> Result<()> {
        let mut socket = self.socket.lock().await;
        let msg = std::str::from_utf8(&state).expect("json is valid text");
        socket
            .send(Message::text(msg))
            .await
            .map_err(|x| anyhow::Error::msg("Connection broke on send"))
    }
    async fn socket_error() {
        panic!("Connection broke on read");
    }
}
