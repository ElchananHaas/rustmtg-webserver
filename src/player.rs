use crate::ent_maps::EntMap;
use crate::entities::{CardId, ManaId, PlayerId};
use crate::game::Cards;
use crate::mana::Mana;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_derive::Serialize;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

use std::time::Duration;
use tokio::time::sleep;
use warp::filters::ws::Message;
use warp::ws::WebSocket;
#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub life: i64,
    pub library: Vec<CardId>,
    pub hand: HashSet<CardId>,
    pub mana_pool: HashSet<ManaId>,
    pub graveyard: Vec<CardId>,
    pub max_handsize: usize,
    pub player_con: PlayerCon,
}

#[derive(Serialize)]
pub struct PlayerView<'a> {
    pub name: &'a str,
    pub life: i64,
    pub library: Vec<Option<CardId>>,
    pub hand: Vec<Option<CardId>>,
    pub graveyard: &'a Vec<CardId>,
    pub mana_pool: &'a HashSet<ManaId>,
    pub max_handsize: usize,
}
fn view_t<'a>(
    cards: &'a Cards,
    r: impl Iterator<Item = &'a CardId> + 'a,
    pl: PlayerId,
    hidden_map: &'a HashMap<CardId, CardId>,
) -> impl Iterator<Item = Option<CardId>> + 'a {
    r.map(move |&id| {
        cards
            .get(id)
            .map(|ent| {
                if ent.known_to.contains(&pl) {
                    Some(id)
                } else {
                    hidden_map.get(&id).map(|x| *x)
                }
            })
            .flatten()
    })
    .filter(|x| x.is_some())
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

    pub async fn send_state(&self, buffer: Vec<u8>) -> Result<()> {
        self.player_con.send_state(buffer).await
    }
    //Select n entities from a vector, returns selected indicies
    pub async fn ask_user_selectn<T: Serialize + Clone>(
        &self,
        ents: &Vec<T>,
        min: u32,
        max: u32,
        reason: AskReason,
    ) -> Vec<usize> {
        let query = AskUser {
            asktype: AskType::SelectN {
                ents: ents.clone(),
                min,
                max,
            },
            reason,
        };
        loop {
            let response = self.player_con.ask_user::<Vec<usize>, T>(&query).await;
            let response_unique: HashSet<usize> = response.iter().cloned().collect();
            if response.len() < min.try_into().unwrap()
                || response.len() > max.try_into().unwrap()
                || response.len() != response_unique.len()
                || response.iter().any(|&i| i >= ents.len())
            {
                continue;
            }
            return response;
        }
    }
    //pair attackers with blockers/attacking targets
    //Returns an adjacency list with either the
    //planeswalker/player each attacker is attacking,
    //or the list of creatures each blocker is blocking
    pub async fn ask_user_pair<T: Clone + Eq + DeserializeOwned + Hash + Copy + Serialize>(
        &self,
        a: Vec<CardId>,
        b: Vec<T>,
        //Min and max number of choices
        num_choices: Vec<(usize, usize)>,
        reason: AskReason,
    ) -> Vec<Vec<T>> {
        let query = AskUser {
            asktype: AskType::PairAB {
                a: a.clone(),
                b: b.clone(),
                num_choices: num_choices.clone(),
            },
            reason,
        };
        'outer: loop {
            let response = self.player_con.ask_user::<Vec<Vec<T>>, T>(&query).await;
            if response.len() != a.len() {
                continue 'outer;
            }
            for item in response.iter().flatten() {
                if !b.contains(item) {
                    continue 'outer;
                }
            }
            for (i, row) in response.iter().enumerate() {
                if row.len() < num_choices[i].0 || row.len() > num_choices[i].1 {
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

#[derive(Clone, Debug, Serialize)]
pub struct AskUser<T> {
    reason: AskReason,
    asktype: AskType<T>,
}
#[derive(Clone, Debug, Serialize)]
pub enum AskType<T> {
    SelectN {
        ents: Vec<T>,
        min: u32,
        max: u32,
    },
    PairAB {
        a: Vec<CardId>,
        b: Vec<T>,
        num_choices: Vec<(usize, usize)>,
    },
}
#[derive(Copy, Clone, Debug, Serialize)]
pub enum AskReason {
    Attackers,
    Blockers,
    DiscardToHandSize,
    Action,
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

    pub async fn ask_user<T: DeserializeOwned, U: Serialize>(&self, query: &AskUser<U>) -> T {
        let mut socket = self.socket.lock().await;
        let res = self.ask_user_socket::<T, U>(query, &mut socket).await;
        res
    }
    async fn ask_user_socket<T: DeserializeOwned, U: Serialize>(
        &self,
        query: &AskUser<U>,
        socket: &mut WebSocket,
    ) -> T {
        let mut buffer = Vec::<u8>::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut json_serial = serde_json::Serializer::new(cursor);
            (&query.reason, &query.asktype)
                .serialize(&mut json_serial)
                .expect("State must be serializable");
        }
        loop {
            let msg = std::str::from_utf8(&buffer).expect("json is valid text");
            let sres = socket.send(Message::text(msg)).await;

            if sres.is_err() {
                PlayerCon::socket_error().await;
                continue;
            };
            let recieved = socket.next().await.expect("Socket is still open");
            let message = if let Ok(msg) = recieved {
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
    pub async fn send_state(&self, state: Vec<u8>) -> Result<()> {
        let mut socket = self.socket.lock().await;
        let res = self.send_state_socket(state, &mut socket).await;
        res
    }
    async fn send_state_socket(&self, state: Vec<u8>, socket: &mut WebSocket) -> Result<()> {
        let msg = std::str::from_utf8(&state).expect("json is valid text");
        socket.send(Message::text(msg)).await?;
        Ok(())
    }
    async fn socket_error() {
        panic!("Connection to client broken"); //Give up after around 5 min
    }
}
