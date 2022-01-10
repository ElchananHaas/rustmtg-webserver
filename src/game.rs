use crate::ability::Ability;
use crate::ability::KeywordAbility;
use crate::carddb::CardDB;
use crate::components::{
    CardName, Controller, EntCore, Subtype, SummoningSickness, Tapped, Types, PT,
};
use crate::event::{Event, EventCause, EventResult, TagEvent};
use crate::player::{Player, PlayerCon, PlayerSerialHelper};
use anyhow::{bail, Result};
use futures::{executor, future, FutureExt};
use hecs::serialize::row::{try_serialize, SerializeContext};
use hecs::{Entity, EntityRef, World,EntityBuilder};
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;
use serde_derive::Serialize;
use serde_json;
use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::ops::Sub;
use std::rc::Rc;
use std::result::Result::Ok;
use std::sync::Arc;
use std::sync::Mutex;

mod handle_event;

macro_rules! backuprestore {
    ( $( $x:ty ),* ) => {
        fn copy_simple_comp(source:&mut World,dest:&mut World){
            let mut build=EntityBuilder::new();
            for entref in source.iter(){
            $(
                if let Some(comp)=entref.get::<$x>(){
                    build.add((*comp).clone());
                }
            )*
            let entid=entref.entity();
            if(!dest.contains(entid)){
                dest.spawn_at(entid, build.build());
            }else{
                dest.insert(entid,build.build());
            }
            }
        }
        fn clear_comp(ents: &mut World){
            let entids=ents.iter().map(|entref| entref.entity() ).collect::<Vec<Entity>>();
            for id in entids{
            $(
                let _=ents.remove_one::<$x>(id);
            )*
            }
        }

    };
}

backuprestore!{Tapped,SummoningSickness,Player}

pub struct GameBuilder {
    ents: World,
    turn_order: Vec<Entity>,
    active_player: Option<Entity>,
}
//Implement debug trait!
//Implement clone trait???
#[derive(Serialize)]
pub struct Game {
    #[serde(skip_serializing)]
    pub ents: World,
    pub battlefield: HashSet<Entity>,
    pub exile: HashSet<Entity>,
    pub command: HashSet<Entity>,
    pub stack: Vec<Entity>,
    pub turn_order: Vec<Entity>,
    pub extra_turns: VecDeque<Entity>,
    pub phases: VecDeque<Phase>,
    pub subphases: VecDeque<Subphase>,
    pub phase: Option<Phase>,
    pub subphase: Option<Subphase>,
    pub active_player: Entity,
    pub outcome: GameOutcome,
    #[serde(skip_serializing)]
    db: &'static CardDB,
    #[serde(skip_serializing)]
    backup: Option<BackupGame>,
}

pub struct BackupGame{
    pub ents: World,
    pub battlefield: HashSet<Entity>,
    pub exile: HashSet<Entity>,
    pub command: HashSet<Entity>,
    pub stack: Vec<Entity>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum GameOutcome {
    Ongoing,
    Tie,
    Winner(Entity),
}
impl GameBuilder {
    pub fn new() -> Self {
        GameBuilder {
            ents: World::new(),
            turn_order: Vec::new(),
            active_player: None,
        }
    }
    //If this function fails the game is corrupted
    pub fn add_player(
        &mut self,
        name: &str,
        db: &CardDB,
        card_names: &Vec<String>,
        player_con: Box<dyn PlayerCon>,
    ) -> Result<Entity> {
        let mut cards = Vec::new();
        let player = Player {
            name: name.to_owned(),
            hand: HashSet::new(),
            life: 20,
            mana_pool: HashSet::new(),
            graveyard: Vec::new(),
            lost: false,
            won: false,
            deck: Vec::new(),
            player_con: Arc::new(Mutex::new(player_con)),
        };
        let player: Entity = self.ents.spawn((player,));
        for cardname in card_names {
            let card: Entity = db.spawn_card(&mut self.ents, &cardname, player)?;
            cards.push(card);
        }
        //Now that the deck has been constructed, set the players deck
        self.ents.get_mut::<Player>(player).unwrap().deck = cards;
        self.turn_order.push(player);
        if self.active_player.is_none() {
            self.active_player = Some(player);
        }
        Ok(player)
    }
    pub fn build(self, db: &'static CardDB) -> Result<Game> {
        let active_player = match self.active_player {
            Some(player) => player,
            None => {
                bail!("Active player must be set in game initilization");
            }
        };
        if self.turn_order.len() < 2 {
            bail!("Game needs at least two players in initialization")
        };
        Ok(Game {
            ents: self.ents,
            battlefield: HashSet::new(),
            exile: HashSet::new(),
            command: HashSet::new(),
            stack: Vec::new(),
            turn_order: self.turn_order,
            active_player: active_player,
            db: db,
            extra_turns: VecDeque::new(),
            phases: VecDeque::new(),
            subphases: VecDeque::new(),
            phase: None,
            subphase: None,
            outcome: GameOutcome::Ongoing,
            backup: None,
        })
    }
}
//This structure serializes a game from the view of
//the player for sending to the client
struct GameSerializer<'a> {
    player: Entity,
    ents: &'a World,
}
impl<'a> SerializeContext for GameSerializer<'a> {
    fn serialize_entity<S>(&mut self, entity: EntityRef<'_>, map: &mut S) -> Result<(), S::Error>
    where
        S: serde::ser::SerializeMap,
    {
        let toshow = match entity.get::<EntCore>() {
            Some(core) => core.known.contains(&self.player),
            None => true,
        };
        if toshow {
            try_serialize::<SummoningSickness, _, _>(&entity, "summoning_sickness", map)?;
            try_serialize::<Tapped, _, _>(&entity, "tapped", map)?;
            try_serialize::<CardName, _, _>(&entity, "name", map)?;
            try_serialize::<EntCore, _, _>(&entity, "base_identity", map)?;
            try_serialize::<PT, _, _>(&entity, "pt", map)?;
            try_serialize::<Types, _, _>(&entity, "types", map)?;
            try_serialize::<HashSet<Subtype>, _, _>(&entity, "subtypes", map)?;
        }
        if let Some(pl) = entity.get::<Player>() {
            let helper = PlayerSerialHelper {
                viewpoint: self.player,
                player: &pl,
                world: self.ents,
            };
            map.serialize_entry("player", &helper)?;
        }
        Ok(())
    }
}
impl Game {
    pub async fn run(&mut self) -> GameOutcome {
        for player in self.turn_order.clone() {
            for _i in 0..7 {
                self.draw(player, EventCause::None);
            }
        }
        self.send_state().await;
        while self.outcome == GameOutcome::Ongoing {
            if let Some(subphase) = self.subphases.pop_front() {
                continue;
            }
            if let Some(phase) = self.phases.pop_front() {
                continue;
            }
        }
        self.outcome
    }
    async fn send_state(&mut self) {
        let mut state_futures = Vec::new();
        for player in self.turn_order.clone() {
            state_futures.push(self.send_state_player(player));
        }
        let _results = future::join_all(state_futures).await;
    }
    async fn send_state_player(&self, player: Entity) -> Result<()> {
        let mut buffer = Vec::<u8>::new();
        {
            let mut cursor = std::io::Cursor::new(&mut buffer);
            cursor.write_all(b"[")?;
            let mut json_serial = serde_json::Serializer::new(cursor);
            let mut serial_context = GameSerializer {
                player: player,
                ents: &self.ents,
            };
            hecs::serialize::row::serialize(&self.ents, &mut serial_context, &mut json_serial)?;
            let mut cursor = json_serial.into_inner();
            cursor.write_all(b",")?;
            let mut json_serial = serde_json::Serializer::new(cursor);
            self.serialize_game(&mut json_serial, player)?;
            let mut cursor = json_serial.into_inner();
            cursor.write_all(b"]")?;
        }
        if let Ok(mut pl) = self.ents.get_mut::<Player>(player) {
            pl.send_state(buffer).await?;
        }
        Ok(())
    }
    //This function will be needed later for face-down cards. For now,
    //Just show all information for these entities
    fn serialize_game<W: std::io::Write>(
        &self,
        S: &mut serde_json::Serializer<W>,
        player: Entity,
    ) -> Result<()> {
        let mut sergame = S.serialize_struct("game", 6)?;
        sergame.serialize_field("exile", &self.exile)?;
        sergame.serialize_field("command", &self.command)?;
        sergame.serialize_field("stack", &self.stack)?;
        sergame.serialize_field("turn_order", &self.turn_order)?;
        sergame.serialize_field("active_player", &self.active_player)?;
        sergame.serialize_field("outcome", &self.outcome)?;
        sergame.end()?;
        Ok(())
    }

    //backs the game up in case a spell casting or attacker/blocker
    //declaration fails. Only backs up what is needed
    pub fn backup(&mut self){
        let mut bup=BackupGame{
            ents:World::new(),
            battlefield:self.battlefield.clone(),
            exile:self.exile.clone(),
            command:self.command.clone(),
            stack:self.stack.clone(),
        };
        copy_simple_comp(&mut self.ents,&mut bup.ents);
        self.backup=Some(bup);
    }
    pub fn restore(&mut self){
        let mut bup=None;
        std::mem::swap(&mut bup, &mut self.backup);
        let mut bup=bup.expect("Game must already be backed up!");
        self.battlefield=bup.battlefield;
        self.exile=bup.exile;
        self.command=bup.command;
        self.stack=bup.stack;
        clear_comp(&mut self.ents);
        copy_simple_comp(&mut bup.ents, &mut self.ents);
    }
    fn add_event(events: &mut Vec<TagEvent>, event: Event, cause: EventCause) {
        events.push(TagEvent {
            event: event,
            replacements: Vec::new(),
            cause: cause,
        });
    }
    //Taps an entity, returns if it was sucsessfully tapped
    pub fn tap(&mut self, ent: Entity, cause: EventCause) -> bool {
        self.handle_event(Event::Tap { ent }, cause)
            .contains(&EventResult::Tap(ent))
    }
    //Taps an entity, returns if it was sucsessfully tapped
    pub fn untap(&mut self, ent: Entity, cause: EventCause) -> bool {
        self.handle_event(Event::Untap { ent }, cause)
            .contains(&EventResult::Untap(ent))
    }
    //draws a card, returns the entities drawn
    pub fn draw(&mut self, player: Entity, cause: EventCause) -> Vec<Entity> {
        let res = self.handle_event(
            Event::Draw {
                player: player,
                controller: None,
            },
            cause,
        );
        let drawn = Vec::new();
        drawn
        //TODO figure out which cards were drawn!
    }
    pub fn players_creatures(&self, player: Entity) -> Vec<Entity> {
        let mut also_creature = Vec::new();
        for ent in self.controlled(player) {
            if let Ok(types) = self.ents.get::<Types>(ent) {
                if types.creature {
                    also_creature.push(ent);
                }
            }
        }
        also_creature
    }
    pub fn controlled(&self, player: Entity) -> Vec<Entity> {
        let mut controlled = Vec::new();
        for perm in self.battlefield.clone() {
            if self.get_controller(perm) == Some(player) {
                controlled.push(perm);
            }
        }
        controlled
    }
    //Can this creature tap to be declared an attacker or to activate an ability?
    //Doesn't include prevention effects, just if it can tap w/o them
    pub fn can_tap(&self, ent: Entity) -> bool {
        if self.ents.get::<Tapped>(ent).is_ok() {
            return false;
        }
        if let Ok(types) = self.ents.get::<Types>(ent) {
            if types.creature {
                if self.ents.get::<SummoningSickness>(ent).is_ok() {
                    if let Ok(abilities) = self.ents.get::<Vec<Ability>>(ent) {
                        for ability in &(*abilities) {
                            if ability.keyword() == Some(KeywordAbility::Haste) {
                                return true;
                            }
                        }
                        return false;
                    } else {
                        false
                    }
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            false
        }
    }
    //takes in a card or permanent, returns it's controller or owner if the controller
    //is unavailable
    pub fn get_controller(&self, ent: Entity) -> Option<Entity> {
        if let Ok(controller) = self.ents.get::<Controller>(ent) {
            Some(controller.0)
        } else {
            if let Ok(core) = self.ents.get::<EntCore>(ent) {
                Some(core.owner)
            } else {
                None
            }
        }
    }
    //Cycles priority then fires an end-of-phase event
    pub fn run_phase(&mut self) {
        todo!();
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[allow(dead_code)] //allow dead code to reduce warnings noise on each variant

pub enum Phase {
    Begin,
    FirstMain,
    Combat,
    SecondMain,
    Ending,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[allow(dead_code)]
pub enum Subphase {
    Untap,
    Upkeep,
    Draw,
    BeginCombat,
    Attackers,
    Blockers,
    FirstStrikeDamage,
    Damage,
    EndCombat,
    EndStep,
    Cleanup,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)] //allow dead code to reduce warnings noise on each variant
pub enum Zone {
    Hand,
    Library,
    Exile,
    Battlefield,
    Graveyard,
    Command,
    Stack,
}
