use crate::ability::Ability;
use crate::ability::KeywordAbility;
use crate::carddb::CardDB;
use crate::components::Attacking;
use crate::components::Blocked;
use crate::components::Blocking;
use crate::components::{
    CardName, Controller, EntCore, Subtype, SummoningSickness, Tapped, Types, PT,
};
use crate::event::DiscardCause;
use crate::event::{Event, EventResult, TagEvent};
use crate::player::{Player, PlayerCon, PlayerSerialHelper};
use anyhow::{bail, Result};
use futures::future;
use hecs::serialize::row::{try_serialize, SerializeContext};
use hecs::Component;
use hecs::{Entity, EntityBuilder, EntityRef, World};
use serde::Serialize;
use serde_derive::Serialize;
use serde_json;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::Write;
use std::result::Result::Ok;
use std::sync::Arc;
use warp::ws::WebSocket;

mod handle_event;

macro_rules! backuprestore {
    ( $( $x:ty ),* ) => {
        fn copy_simple_comp(source:& World,dest:&mut World){
            let mut build=EntityBuilder::new();
            for entref in source.iter(){
            $(
                if let Some(comp)=entref.get::<$x>(){
                    build.add((*comp).clone());
                }
            )*
            let entid=entref.entity();
            if(dest.contains(entid)){
                let _=dest.insert(entid,build.build());
            }else{
                dest.spawn_at(entid, build.build());
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

backuprestore! {Tapped,Player,Attacking,Blocked,Blocking}

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

pub struct BackupGame {
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
        player_con: WebSocket,
    ) -> Result<Entity> {
        let mut cards = Vec::new();
        let psocket = tokio::sync::Mutex::new(PlayerCon::new(player_con));
        let player = Player {
            name: name.to_owned(),
            hand: HashSet::new(),
            life: 20,
            mana_pool: HashSet::new(),
            graveyard: Vec::new(),
            lost: false,
            won: false,
            deck: Vec::new(),
            max_handsize: 7,
            player_con: Arc::new(psocket),
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
                self.draw(player).await;
            }
        }
        self.send_state().await;
        while self.outcome == GameOutcome::Ongoing {
            if let Some(subphase) = self.subphases.pop_front() {
                self.handle_event(Event::Subphase { subphase }).await;
            } else if let Some(phase) = self.phases.pop_front() {
                self.handle_event(Event::Phase { phase }).await;
            } else if let Some(player) = self.extra_turns.pop_front() {
                self.handle_event(Event::Turn {
                    player,
                    extra: true,
                })
                .await;
            } else {
                //Make sure the active player is updated properly if a player loses or leaves!
                let order_spot = self
                    .turn_order
                    .iter()
                    .position(|x| *x == self.active_player)
                    .unwrap();
                let new_spot = (order_spot + 1) % self.turn_order.len();
                let player = self.turn_order[new_spot];
                self.handle_event(Event::Turn {
                    player,
                    extra: false,
                })
                .await;
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
            self.serialize(&mut json_serial)?;
            let mut cursor = json_serial.into_inner();
            cursor.write_all(b"]")?;
        }
        if let Ok(mut pl) = self.ents.get_mut::<Player>(player) {
            pl.send_state(buffer).await?;
        }
        Ok(())
    }

    //backs the game up in case a spell casting or attacker/blocker
    //declaration fails. Only backs up what is needed
    pub fn backup(&mut self) {
        let mut bup = BackupGame {
            ents: World::new(),
            battlefield: self.battlefield.clone(),
            exile: self.exile.clone(),
            command: self.command.clone(),
            stack: self.stack.clone(),
        };
        copy_simple_comp(&mut self.ents, &mut bup.ents);
        self.backup = Some(bup);
    }
    pub fn restore(&mut self) {
        let bup = (self.backup)
            .as_ref()
            .expect("Game must already be backed up!");
        self.battlefield = bup.battlefield.clone();
        self.exile = bup.exile.clone();
        self.command = bup.command.clone();
        self.stack = bup.stack.clone();
        clear_comp(&mut self.ents);
        copy_simple_comp(&bup.ents, &mut self.ents);
    }
    //Taps an entity, returns if it was sucsessfully tapped
    pub async fn tap(&mut self, ent: Entity) -> bool {
        self.handle_event(Event::Tap { ent })
            .await
            .contains(&EventResult::Tap(ent))
    }
    //Taps an entity, returns if it was sucsessfully tapped
    pub async fn untap(&mut self, ent: Entity) -> bool {
        self.handle_event(Event::Untap { ent })
            .await
            .contains(&EventResult::Untap(ent))
    }
    //draws a card, returns the entities drawn
    pub async fn draw(&mut self, player: Entity) -> Vec<Entity> {
        let res = self.handle_event(Event::Draw { player }).await;
        let drawn = Vec::new();
        drawn
        //TODO figure out which cards were drawn!
    }
    //discard cards, returns discarded cards
    pub async fn discard(
        &mut self,
        player: Entity,
        card: Entity,
        cause: DiscardCause,
    ) -> Vec<Entity> {
        let res = self
            .handle_event(Event::Discard {
                player,
                card,
                cause,
            })
            .await;
        let discarded = Vec::new();
        discarded
        //TODO figure out which cards were discarded!
    }
    pub fn players_creatures<'b>(&'b self, player: Entity) -> impl Iterator<Item = Entity> + 'b {
        self.all_creatures()
            .into_iter()
            .filter(move |&ent| self.get_controller(ent) == Some(player))
    }
    pub fn players_permanents<'b>(&'b self, player: Entity) -> impl Iterator<Item = Entity> + 'b {
        self.battlefield
            .clone()
            .into_iter()
            .filter(move |&ent| self.get_controller(ent) == Some(player))
    }
    pub fn all_creatures<'b>(&'b self) -> impl Iterator<Item = Entity> + 'b {
        self.battlefield.clone().into_iter().filter(move |&ent| {
            if let Ok(types) = self.ents.get::<Types>(ent) {
                types.creature
            } else {
                false
            }
        })
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
    pub fn cycle_priority(&mut self) {
        todo!();
    }
    pub fn attack_targets(&self, player: Entity) -> Vec<Entity> {
        self.opponents(player)
    }
    pub fn opponents(&self, player: Entity) -> Vec<Entity> {
        self.turn_order
            .iter()
            .filter_map(|x| if *x == player { Some(*x) } else { None })
            .collect()
    }
    //Checks if this attacking arragment is legal.
    //Does nothing for now, will need to implement legality
    //checking before I can make any progress on that
    pub fn attackers_legal(&self,attackers:&Vec<Entity>, targets: &Vec<Entity>) -> bool {
        true
    }
    pub fn blocks_legal(&self,blockers:&Vec<Entity>, blocked: &Vec<Vec<Entity>>) -> bool {
        true
    }
    fn has<T: Component>(&self, ent: Entity) -> bool {
        self.ents.get::<T>(ent).is_ok()
    }
    fn lacks<T: Component>(&self, ent: Entity) -> bool {
        self.ents.get::<T>(ent).is_err()
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
