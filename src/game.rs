use crate::AppendableMap::{EntMap};
use crate::ability::Ability;
use crate::card_entities::{PT, CardEnt};
use crate::carddb::CardDB;
use crate::components::{Attacking, Blocked, Blocking, Damage};
use crate::components::{
    CardName, Controller, EntCore, Subtype, SummoningSickness, Tapped,
};
use crate::entities::{PlayerId, CardId};
use crate::event::{DiscardCause, Event, EventResult, TagEvent};
use crate::mana::{ManaCostSymbol, Color, Mana};
use crate::player::{Player, PlayerCon, PlayerSerialHelper};
use crate::spellabil::KeywordAbility;
use anyhow::{bail, Result};
use futures::future;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_derive::Serialize;
use serde_json;
use std::cmp::max;
use std::collections::{HashSet, VecDeque};
use warp::ws::WebSocket;
mod handle_event;
mod layers;


pub type Players=EntMap<PlayerId,Player>;
pub type Cards=EntMap<CardId,CardEnt>;
pub struct GameBuilder {
    players:Players,
    cards:Cards,
    turn_order: Vec<PlayerId>,
    active_player: Option<PlayerId>,
}
//Implement debug trait!
#[derive(Serialize)]
pub struct Game {
    #[serde(skip_serializing)]
    pub players: Players,
    #[serde(skip_serializing)]
    pub cards: Cards,
    pub battlefield: HashSet<CardId>,
    pub exile: HashSet<CardId>,
    pub command: HashSet<CardId>,
    pub stack: Vec<CardId>,
    pub turn_order: Vec<PlayerId>,
    pub extra_turns: VecDeque<PlayerId>,
    pub phases: VecDeque<Phase>,
    pub subphases: VecDeque<Subphase>,
    pub phase: Option<Phase>,
    pub subphase: Option<Subphase>,
    pub active_player: PlayerId,
    pub outcome: GameOutcome,
    #[serde(skip_serializing)]
    db: &'static CardDB,
    #[serde(skip_serializing)]
    backup: Option<BackupGame>,
    #[serde(skip_serializing)]
    rng: rand::rngs::StdRng, //Store the RNG to allow for deterministic replay
                             //if I choose to implement it
}

pub struct BackupGame {
    pub players: EntMap<PlayerId,Player>,
    pub cards: EntMap<CardId,CardEnt>,
    pub battlefield: HashSet<CardId>,
    pub exile: HashSet<CardId>,
    pub command: HashSet<CardId>,
    pub stack: Vec<CardId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum GameOutcome {
    Ongoing,
    Tie,
    Winner(PlayerId),
}
impl GameBuilder {
    pub fn new() -> Self {
        GameBuilder {
            players: Players::new(),
            cards: Cards::new(),
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
    ) -> Result<PlayerId> {
        let mut cards = Vec::new();
        let player = Player {
            name: name.to_owned(),
            hand: HashSet::new(),
            life: 20,
            mana_pool: HashSet::new(),
            graveyard: Vec::new(),
            lost: false,
            won: false,
            library: Vec::new(),
            max_handsize: 7,
            player_con: PlayerCon::new(player_con),
        };
        let player_id: PlayerId = self.ents.spawn((player,));
        for cardname in card_names {
            let card: CardId = db.spawn_card(&mut self.ents, &cardname, player);
            cards.push(card);
        }
        //Now that the deck has been constructed, set the players deck
        self.ents.get_mut::<Player>(player).unwrap().library = cards;
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
            players: self.players,
            cards: self.cards,
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
            rng: rand::rngs::StdRng::from_entropy(),
        })
    }
}
//This structure serializes a game from the view of
//the player for sending to the client
struct CardSerializer<'a> {
    viewpoint: PlayerId,
    cards: &'a Cards,
}
struct PlayerSerializer<'a> {
    viewpoint: PlayerId,
    players: &'a Players,
}


//This is a helper struct for game serialization because
//the function takes a mutable context,
//so serialize needs to be implemented on a different struct

impl<'a> Serialize for CardSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for (k,v) in self.cards.ser_view(){
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}
impl Game {
    pub async fn run(&mut self) -> GameOutcome {
        for player in self.turn_order.clone() {
            self.shuffle(player);
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
    async fn send_state_player(&self, player: PlayerId) -> Result<()> {
        let serial_context = GameSerialContext {
            player: player,
            ents: &self.ents,
        };
        let mut buffer = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut json_serial = serde_json::Serializer::new(cursor);
            let added_context = ("GameState", player, serial_context, self);
            added_context.serialize(&mut json_serial)?;
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
    pub fn shuffle(&mut self, player: Entity) {
        if let Ok(mut pl) = self.ents.get_mut::<Player>(player) {
            pl.library.shuffle(&mut self.rng);
        }
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
    pub async fn add_mana(&mut self,player:Entity,mana:ManaCostSymbol)->Result<Entity>{
        let color:Color=match mana{
            ManaCostSymbol::Black=>Color::Black,
            ManaCostSymbol::Blue=>Color::Blue,
            ManaCostSymbol::Green=>Color::Green,
            ManaCostSymbol::Red=>Color::Red,
            ManaCostSymbol::White=>Color::White,
            ManaCostSymbol::Generic=>Color::Colorless,
            ManaCostSymbol::Colorless=>Color::Colorless
        };
        {//If there is no player, avoid leaking memory by not spawining the mana
            self.ents.get::<Player>(player)?;
        }
        // Handle snow mana later
        let mana=self.ents.spawn((Mana(color),));
        let mut player=self.ents.get_mut::<Player>(player)?;
        player.mana_pool.insert(mana);
        Ok(mana)
    }
    pub fn players_creatures<'b>(&'b self, player: Entity) -> impl Iterator<Item = Entity> + 'b {
        self.all_creatures()
            .into_iter()
            .filter(move |&ent| self.get_controller(ent).ok() == Some(player))
    }
    pub fn ents_and_zones(&self) -> Vec<(Entity, Zone)> {
        let mut res = Vec::new();
        res.extend(
            self.battlefield
                .iter()
                .cloned()
                .map(|x| (x, Zone::Battlefield)),
        );
        res.extend(self.stack.iter().cloned().map(|e| (e, Zone::Stack)));
        res.extend(self.exile.iter().cloned().map(|e| (e, Zone::Exile)));
        res.extend(self.command.iter().cloned().map(|e| (e, Zone::Command)));
        for player_id in self.turn_order.clone() {
            if let Ok(player) = self.ents.get::<Player>(player_id) {
                res.extend(player.hand.iter().cloned().map(|e| (e, Zone::Hand)));
                res.extend(
                    player
                        .graveyard
                        .iter()
                        .cloned()
                        .map(|e| (e, Zone::Graveyard)),
                );
                res.extend(player.library.iter().cloned().map(|e| (e, Zone::Library)))
            }
        }
        res
    }
    pub fn players_permanents<'b>(&'b self, player: Entity) -> impl Iterator<Item = Entity> + 'b {
        self.battlefield
            .clone()
            .into_iter()
            .filter(move |&ent| self.get_controller(ent).ok() == Some(player))
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
                    self.has_keyword(ent, KeywordAbility::Haste)
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
    pub fn get_controller(&self, ent: Entity) -> Result<Entity> {
        if let Ok(controller) = self.ents.get::<Controller>(ent) {
            Ok(controller.0)
        } else {
            if let Ok(core) = self.ents.get::<EntCore>(ent) {
                Ok(core.owner)
            } else {
                bail!("No controller or owner");
            }
        }
    }
    pub async fn cycle_priority(&mut self) {
        self.place_abilities().await;
        for player in self.turn_order.clone() {
            self.grant_priority(player).await;
        }
    }
    pub async fn grant_priority(&mut self, player: Entity) {
        self.layers();
        //TODO actually grant priority
    }
    //Places abilities on the stack
    pub async fn place_abilities(&mut self) {
        //TODO make this do something!
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
    pub fn attackers_legal(&self, attackers: &Vec<Entity>, targets: &Vec<Entity>) -> bool {
        true
    }
    pub fn blocks_legal(&self, blockers: &Vec<Entity>, blocked: &Vec<Vec<Entity>>) -> bool {
        true
    }
    fn has<T: Component>(&self, ent: Entity) -> bool {
        self.ents.get::<T>(ent).is_ok()
    }
    pub fn remaining_lethal(&self, ent: Entity) -> Option<i32> {
        if let Ok(pt) = self.ents.get::<PT>(ent) {
            if let Ok(damage) = self.ents.get::<Damage>(ent) {
                Some(max(pt.toughness - damage.0, 0))
            } else {
                Some(pt.toughness)
            }
        } else {
            None
        }
    }
    pub fn has_keyword(&self, ent: Entity, keyword: KeywordAbility) -> bool {
        if let Ok(abilities) = self.ents.get::<Vec<Ability>>(ent) {
            !abilities.iter().any(|abil| abil.keyword() == Some(keyword))
        } else {
            false
        }
    }
    //Allow cards to use get, but not get_mut
    pub fn get<'a, T: Component>(
        &'a self,
        ent: Entity,
    ) -> Result<hecs::Ref<'a, T>, hecs::ComponentError> {
        self.ents.get::<T>(ent)
    }
    pub fn add_ability(&self, ent: Entity, ability: Ability) {
        //Assume the builder has already added a vector of abilities
        if let Ok(mut abils) = self.ents.get_mut::<Vec<Ability>>(ent) {
            abils.push(ability);
            return;
        }
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
