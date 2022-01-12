use crate::components::EntCore;
use crate::JS_UNKNOWN;
use anyhow::{bail, Result};
use derivative::*;
use futures::StreamExt;
//derivative::Derivative, work around rust-analyzer bug for now
use futures_util::SinkExt;
use hecs::{Entity, World};
use serde::de::DeserializeOwned;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_derive::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
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
    pub max_handsize: usize,
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
            let mut lock = self.player_con.lock().await;
            let res = lock.ask_user::<HashSet<Entity>>(&query).await;
            if let Ok(response) = res {
                if response.len() < min.try_into().unwrap()
                    && response.len() >= max.try_into().unwrap()
                {
                    continue;
                }
                return response;
            }
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
            let mut lock = self.player_con.lock().await;
            let res = lock.ask_user::<Vec<Vec<Entity>>>(&query).await;
            if let Ok(response) = res {
                if response.len() != a.len() {
                    continue 'outer;
                }
                for item in response.iter().flatten() {
                    if !b.contains(item) {
                        continue 'outer;
                    }
                }
                for (i, row) in response.iter().enumerate() {
                    if row.len() < num_choices[i].0 || row.len() >= num_choices[i].1 {
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
            query.serialize(&mut json_serial)?;
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
