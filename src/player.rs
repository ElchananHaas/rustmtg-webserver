use crate::components::EntCore;
use crate::JS_UNKNOWN;
use anyhow::{bail, Result};
use async_trait::async_trait;
use derivative::*;
use futures::StreamExt;
//derivative::Derivative, work around rust-analyzer bug for now
use futures_util::SinkExt;
use hecs::{Entity, EntityRef, World};
use serde::de::DeserializeOwned;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_derive::Serialize;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use warp::filters::ws::Message;
use warp::ws::WebSocket;
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct Player {
    pub name: String,
    pub life: i32,
    pub deck: Vec<Entity>,
    pub hand: HashSet<Entity>,
    pub mana_pool: HashSet<Entity>,
    pub graveyard: Vec<Entity>,
    pub lost: bool,
    pub won: bool,
    #[derivative(Debug = "ignore")]
    pub player_con: Arc<tokio::sync::Mutex<PlayerCon>>,
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
        for card in &self.deck {
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
    pub async fn send_state(&mut self, state: Vec<u8>) -> Result<()> {
        let mut lock = self.player_con.lock().await;
        lock.send_state(state).await
    }
    //Select n entities from a set
    pub async fn ask_user_selectn(
        &mut self,
        ents: Vec<Entity>,
        min: i32,
        max: i32,
        reason: AskReason,
    ) -> Vec<Entity> {
        let query = AskUser {
            asktype: AskType::SelectN { ents, min, max },
            reason,
        };
        loop {
            let mut lock = self.player_con.lock().await;
            let res = lock.ask_user::<Vec<Entity>>(&query).await;
            if let Ok(response) = res {
                if response.len() < min.try_into().unwrap()
                    && response.len() >= max.try_into().unwrap()
                {
                    continue;
                }
                let as_set = response.iter().map(|x| *x).collect::<HashSet<Entity>>();
                if response.len() != as_set.len() {
                    continue;
                }
                return response;
            }
        }
    }
    //pair attackers with blockers/attacking targets
    //Returns an adjacency matrix with either the
    //planeswalker/player each attacker is attacking, or the
    //list of blockers that the attacker is blocked by.
    pub async fn ask_user_pair(
        &mut self,
        a: Vec<Entity>,
        b: Vec<Entity>,
        reason: AskReason,
    ) -> Vec<Vec<Entity>> {
        let query = AskUser {
            asktype: AskType::PairAB {
                a: a.clone(),
                b: b.clone(),
            },
            reason,
        };
        loop {
            let mut lock = self.player_con.lock().await;
            let res = lock.ask_user::<Vec<Vec<Entity>>>(&query).await;
            if let Ok(response) = res {
                if response.len() != a.len() {
                    continue;
                }
                for item in response.iter().flatten() {
                    if !b.contains(item) {
                        continue;
                    }
                }
                for row in response.iter() {
                    let as_set = row.iter().map(|x| *x).collect::<HashSet<Entity>>();
                    if row.len() != as_set.len() {
                        continue;
                    }
                }
                return response;
            }
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
    asktype: AskType,
    reason: AskReason,
}
#[derive(Clone, Debug, Serialize)]
pub enum AskType {
    SelectN {
        ents: Vec<Entity>,
        min: i32,
        max: i32,
    },
    PairAB {
        a: Vec<Entity>,
        b: Vec<Entity>,
    },
}
#[derive(Copy, Clone, Debug, Serialize)]
pub enum AskReason {
    Attackers,
    Blockers,
}

pub struct PlayerCon {
    socket: WebSocket,
}

impl PlayerCon {
    pub fn new(socket: WebSocket) -> Self {
        PlayerCon { socket }
    }
    pub async fn ask_user<T: DeserializeOwned>(&mut self, query: &AskUser) -> Result<T> {
        let mut buffer = Vec::<u8>::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut json_serial = serde_json::Serializer::new(cursor);
            query.serialize(&mut json_serial);
        }
        self.socket.send(Message::binary(buffer)).await?;
        let recieved = self.socket.next().await.expect("Socket is still open")?;
        let text = if let Ok(text) = recieved.to_str() {
            text
        } else {
            bail!("Expected text!");
        };
        let parsed: T = serde_json::from_str(text)?;
        Ok(parsed)
    }
    pub async fn send_state(&mut self, state: Vec<u8>) -> Result<()> {
        self.socket.send(Message::binary(state)).await?;
        Ok(())
    }
}
