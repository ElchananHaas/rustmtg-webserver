use crate::components::EntCore;
use crate::JS_UNKNOWN;
use anyhow::{bail, Result};
use async_trait::async_trait;
use derivative::*; //derivative::Derivative, work around rust-analyzer bug for now
use futures_util::SinkExt;
use hecs::{Entity, EntityRef, World};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::collections::HashSet;
use std::sync::Mutex;
use warp::filters::ws::Message;
use warp::ws::WebSocket;
#[derive(Derivative)]
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
    pub player_con: Box<dyn PlayerCon>,
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

#[async_trait]
pub trait PlayerCon: Send + Sync {
    async fn choose(&mut self, ents: &Vec<Entity>,min:u32,max:u32) -> Result<usize> {
        match ents.len() {
            0 => bail!("Can't choose 0 options"),
            1 => Ok(1),
            _ => Ok(self.ask_user(ents,min,max).await),
        }
    }
    //Inclusive on min, exclusive on max
    async fn ask_user(&mut self, ents: &Vec<Entity>,min:u32,max:u32) -> usize;
    async fn send_state(&mut self, state: Vec<u8>) -> Result<()>;
}

#[async_trait]
impl PlayerCon for Mutex<WebSocket> {
    async fn ask_user(&mut self, ents: &Vec<Entity>,min:u32,max:u32) -> usize {
        0
    }
    async fn send_state(&mut self, state: Vec<u8>) -> Result<()> {
        let socket = self.get_mut().unwrap();
        socket.send(Message::binary(state)).await?;
        Ok(())
    }
}

#[async_trait]
impl PlayerCon for () {
    //Probably would be best to make this random
    async fn ask_user(&mut self, _ents: &Vec<Entity>,_min:u32,_max:u32) -> usize {
        0
    }
    async fn send_state(&mut self, _state: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
