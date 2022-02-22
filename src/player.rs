use crate::components::EntCore;
use crate::JS_UNKNOWN;
use crate::entities::PlayerId;
use anyhow::{bail, Result};
//derivative::Derivative, work around rust-analyzer bug for now
use derivative::*;
use futures::{SinkExt, StreamExt};
use hecs::{Entity, World};
use serde::de::DeserializeOwned;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_derive::Serialize;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use warp::filters::ws::Message;
use warp::ws::WebSocket;
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct Player {
    pub name: String,
    pub life: i32,
    pub library: RefCell<Vec<Entity>>,
    pub hand: RefCell<HashSet<Entity>>,
    pub mana_pool: RefCell<HashSet<Entity>>,
    pub graveyard: RefCell<Vec<Entity>>,
    pub lost: bool,
    pub won: bool,
    pub max_handsize: usize,
    #[derivative(Debug = "ignore")]
    pub player_con: PlayerCon,
}

impl Player {
    fn serialize_view<S>(
        &self,
        serializer: S,
        ents: &World,
        player: Entity,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = serializer.serialize_struct("player", 8)?;
        let mut deckview = Vec::new();
        let library=self.library.borrow();
        for card in *library{
            let core = ents.get::<EntCore>(*card).expect("All cards need a core");
            if core.known.contains(&player) {
                deckview.push(*card);
            } else {
                deckview.push(*JS_UNKNOWN.get().unwrap());
            }
        }
        let mut handview = Vec::new();
        for card in &self.hand {
            let core = ents.get::<EntCore>(*card).expect("All cards need a core");
            if core.known.contains(&player) {
                handview.push(*card);
            } else {
                handview.push(*JS_UNKNOWN.get().unwrap());
            }
        }
        ser.serialize_field("deck", &deckview)?;
        ser.serialize_field("hand", &handview)?;
        ser.serialize_field("name", &self.name)?;
        ser.serialize_field("mana_pool", &self.mana_pool)?;
        ser.serialize_field("graveyard", &self.graveyard)?;
        ser.serialize_field("life", &self.life)?;
        ser.serialize_field("won", &self.won)?;
        ser.serialize_field("lost", &self.lost)?;
        ser.end()
    }
    pub async fn send_state(&mut self, buffer: Vec<u8>) -> Result<()> {
        self.player_con.send_state(buffer).await
    }
    //Select n entities from a set
    pub async fn ask_user_selectn(
        &self,
        ents: &HashSet<Entity>,
        min: i32,
        max: i32,
        reason: AskReason,
    ) -> HashSet<Entity> {
        let query = AskUser {
            asktype: AskType::SelectN {
                ents: ents.clone(),
                min,
                max,
            },
            reason,
        };
        loop {
            let response = self.player_con.ask_user::<HashSet<Entity>>(&query).await;
            if response.len() < min.try_into().unwrap() || response.len() > max.try_into().unwrap()
            {
                continue;
            }
            return response;
        }
    }
    //pair attackers with blockers/attacking targets
    //Returns an adjacency matrix with either the
    //planeswalker/player each attacker is attacking,
    //or the list of creatures each blocker is blocking
    pub async fn ask_user_pair(
        &self,
        a: Vec<Entity>,
        b: Vec<Entity>,
        //Min and max number of choices
        num_choices: Vec<(usize, usize)>,
        reason: AskReason,
    ) -> Vec<Vec<Entity>> {
        let query = AskUser {
            asktype: AskType::PairAB {
                a: a.clone(),
                b: b.clone(),
                num_choices: num_choices.clone(),
            },
            reason,
        };
        'outer: loop {
            let response = self.player_con.ask_user::<Vec<Vec<Entity>>>(&query).await;
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
                let as_set = row.iter().map(|x| *x).collect::<HashSet<Entity>>();
                if row.len() != as_set.len() {
                    continue 'outer;
                }
            }
            return response;
        }
    }
}
pub struct PlayerSerialHelper<'a, 'b> {
    pub viewpoint: Entity,
    pub player: &'a Player,
    pub world: &'b World,
}
impl<'a, 'b> Serialize for PlayerSerialHelper<'a, 'b> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.player
            .serialize_view(serializer, &self.world, self.viewpoint)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AskUser {
    reason: AskReason,
    asktype: AskType,
}
#[derive(Clone, Debug, Serialize)]
pub enum AskType {
    SelectN {
        ents: HashSet<Entity>,
        min: i32,
        max: i32,
    },
    PairAB {
        a: Vec<Entity>,
        b: Vec<Entity>,
        num_choices: Vec<(usize, usize)>,
    },
}
#[derive(Copy, Clone, Debug, Serialize)]
pub enum AskReason {
    Attackers,
    Blockers,
    DiscardToHandSize,
}
#[derive(Clone)]
pub struct PlayerCon {
    socket: Arc<Mutex<Option<WebSocket>>>,
}

impl PlayerCon {
    pub fn new(socket: WebSocket) -> Self {
        PlayerCon {
            socket: Arc::new(Mutex::new(Some(socket))),
        }
    }
    //There should be no contention on this lock,
    //so just take the contents!
    //Can actually be replaced with async lock, that wasn't the problem.
    fn take_socket(&self) -> WebSocket {
        let mut guard = self.socket.lock().unwrap();
        let mut temp = None;
        std::mem::swap(&mut temp, &mut *guard);
        temp.unwrap()
    }
    fn restore_socket(&self, socket: WebSocket) {
        let mut guard = self.socket.lock().unwrap();
        let mut temp = Some(socket);
        std::mem::swap(&mut temp, &mut *guard);
    }
    //This function ensures the socket will be restored, even in the case of an error
    pub async fn ask_user<T: DeserializeOwned>(&self, query: &AskUser) -> T {
        let mut socket = self.take_socket();
        let res = self.ask_user_socket(query, &mut socket).await;
        self.restore_socket(socket);
        res
    }
    async fn ask_user_socket<T: DeserializeOwned>(
        &self,
        query: &AskUser,
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
        let mut failures = 0;
        loop {
            let sres = socket.send(Message::binary(buffer.clone())).await;

            if sres.is_err() {
                PlayerCon::socket_error(&mut failures).await;
                continue;
            };
            let recieved = socket.next().await.expect("Socket is still open");
            let message = if let Ok(msg) = recieved {
                msg
            } else {
                PlayerCon::socket_error(&mut failures).await;
                continue;
            };
            let text = if let Ok(txt) = message.to_str() {
                txt
            } else {
                continue;
            };
            if let Ok(parsed) = serde_json::from_str(text) {
                return parsed;
            } else {
                continue;
            }
        }
    }
    pub async fn send_state(&mut self, state: Vec<u8>) -> Result<()> {
        let mut socket = self.take_socket();
        let res = self.send_state_socket(state, &mut socket).await;
        self.restore_socket(socket);
        res
    }
    async fn send_state_socket(&mut self, state: Vec<u8>, socket: &mut WebSocket) -> Result<()> {
        socket.send(Message::binary(state)).await?;
        Ok(())
    }
    async fn socket_error(failures: &mut u64) {
        let max_failures = 15;
        if *failures > max_failures {
            panic!("Connection to client broken"); //Give up after around 5 min
        } else {
            //Use exponential backoff
            sleep(Duration::from_millis(10 * (*failures).pow(2))).await;
            *failures += 1;
        }
    }
}
