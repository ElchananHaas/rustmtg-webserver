use crate::ability::Ability;
use crate::ability::KeywordAbility;
use crate::carddb::CardDB;
use crate::components::{CardName, Controller, EntCore, SummoningSickness, Tapped, Types, PT,Subtype};
use crate::event::{Event, EventCause, EventResult, TagEvent};
use crate::player::{Player, PlayerCon, PlayerSerialHelper};
use anyhow::{bail, Result};
use futures::{executor, future, FutureExt};
use hecs::serialize::row::{try_serialize, SerializeContext};
use hecs::{Entity, EntityRef, World};
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;
use serde_derive::Serialize;
use serde_json;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::ops::Sub;
use std::result::Result::Ok;
pub struct GameBuilder {
    ents: World,
    turn_order: Vec<Entity>,
    active_player: Option<Entity>,
}
//Implement debug trait!
//Implement clone trait???
#[derive(Serialize)]
pub struct Game<'a> {
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
    db: &'a CardDB,
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
            player_con: player_con,
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
    pub fn build<'a>(self, db: &'a CardDB) -> Result<Game> {
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
        })
    }
}
//This structure serializes a game from the view of
//the playet for sending to the client
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
impl<'a> Game<'a> {
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
            pl.player_con.send_state(buffer).await?;
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
    fn handle_event(&mut self, event: Event, cause: EventCause) -> Vec<EventResult> {
        let mut results: Vec<EventResult> = Vec::new();
        let mut events: Vec<TagEvent> = Vec::new();
        events.push(TagEvent {
            event: event,
            replacements: Vec::new(),
            cause: cause,
        });
        loop {
            let event: TagEvent = match events.pop() {
                Some(x) => x,
                None => {
                    return results;
                }
            };
            //Handle prevention, replacement, triggered abilties here
            //By the time the loop reaches here, the game is ready to
            //Execute the event. No more prevention/replacement effects
            match event.event {
                Event::Turn { extra: _, player } => {
                    self.active_player = player;
                    self.phases.extend(
                        [
                            Phase::Begin,
                            Phase::FirstMain,
                            Phase::Combat,
                            Phase::SecondMain,
                            Phase::Ending,
                        ]
                        .iter(),
                    );
                }
                Event::Subphase { subphase } => {
                    self.subphase = Some(subphase);
                    match subphase {
                        Subphase::Untap => {
                            for perm in self.controlled(self.active_player) {
                                self.untap(perm, EventCause::None);
                            }
                        }
                        Subphase::Upkeep => {
                            self.run_phase();
                        }
                        Subphase::Draw => {
                            self.draw(self.active_player, EventCause::None);
                            self.run_phase();
                        }
                        Subphase::BeginCombat => {
                            self.run_phase();
                        }
                        Subphase::Attackers => {
                            let perms=self.players_creatures(self.active_player);
                            
                            self.run_phase();
                        }
                    }
                }
                Event::Phase { phase } => {
                    self.phase = Some(phase);
                    self.subphase = None;
                    match phase {
                        Phase::Begin => {
                            self.subphases
                                .extend([Subphase::Untap, Subphase::Upkeep, Subphase::Draw].iter());
                        }
                        Phase::FirstMain => {
                            self.run_phase();
                        }
                        Phase::Combat => {
                            self.subphases.extend(
                                [
                                    Subphase::BeginCombat,
                                    Subphase::Attackers,
                                    Subphase::Blockers,
                                    Subphase::FirstStrikeDamage,
                                    Subphase::Damage,
                                    Subphase::EndCombat,
                                ]
                                .iter(),
                            );
                        }
                        Phase::SecondMain => {
                            self.run_phase();
                        }
                        Phase::Ending => {
                            self.subphases
                                .extend([Subphase::EndStep, Subphase::Cleanup].iter());
                        }
                    }
                }
                //Handle already being tapped as prevention effect
                Event::Tap { ent } => {
                    if self.battlefield.contains(&ent)
                        && self.ents.insert_one(ent, Tapped()).is_ok()
                    {
                        results.push(EventResult::Tap(ent));
                    }
                }
                //Handle already being untapped as prevention effect
                Event::Untap { ent } => {
                    if self.battlefield.contains(&ent)
                        && self.ents.remove_one::<Tapped>(ent).is_ok()
                    {
                        results.push(EventResult::Untap(ent));
                    }
                }
                Event::Draw {
                    player,
                    controller: _,
                } => {
                    if let Ok(pl) = self.ents.get::<Player>(player) {
                        match pl.deck.last() {
                            Some(card) => {
                                Game::add_event(
                                    &mut events,
                                    Event::MoveZones {
                                        ent: *card,
                                        origin: Zone::Library,
                                        dest: Zone::Hand,
                                    },
                                    event.cause,
                                );
                                results.push(EventResult::Draw(*card));
                            }
                            None => Game::add_event(
                                &mut events,
                                Event::Lose { player: player },
                                EventCause::Trigger(event.event.clone()),
                            ),
                        }
                    }
                }
                Event::Cast {
                    player: _,
                    spell: _,
                } => {
                    //The spell has already had costs/modes chosen.
                    //this is just handling triggered abilities
                    //So there is nothing to do here.
                    //Spells are handled differently from other actions
                    //Because of the rules complexity
                }
                Event::Activate {
                    controller: _,
                    ability: _,
                } => {
                    //Similar to spell casting
                }
                Event::Lose { player } => {
                    //TODO add in the logic to have the game terminate such as setting winners
                    if let Ok(mut pl) = self.ents.get_mut::<Player>(player) {
                        (*pl).lost = true;
                    }
                }
                Event::MoveZones { ent, origin, dest } => {
                    if origin == dest {
                        continue;
                    };
                    let core = &mut None;
                    {
                        if let Ok(coreborrow) = self.ents.get::<EntCore>(ent) {
                            let refentcore = &(*coreborrow);
                            *core = Some(refentcore.clone());
                        }
                    }
                    let mut removed=false;
                    if let Some(core) = core.as_mut() {
                        removed = if let Ok(mut player) = self.ents.get_mut::<Player>(core.owner) {
                            match origin {
                                Zone::Exile => self.exile.remove(&ent),
                                Zone::Command => self.command.remove(&ent),
                                Zone::Battlefield => self.battlefield.remove(&ent),
                                Zone::Hand => player.hand.remove(&ent),
                                Zone::Library => match player.deck.iter().position(|x| *x == ent) {
                                    Some(i) => {
                                        player.deck.remove(i);
                                        true
                                    }
                                    None => false,
                                },
                                Zone::Graveyard => {
                                    match player.graveyard.iter().position(|x| *x == ent) {
                                        Some(i) => {
                                            player.graveyard.remove(i);
                                            true
                                        }
                                        None => false,
                                    }
                                }
                                Zone::Stack => match self.stack.iter().position(|x| *x == ent) {
                                    Some(i) => {
                                        self.stack.remove(i);
                                        true
                                    }
                                    None => false,
                                },
                            }
                        } else {
                            false
                        }
                    }
                    if let Some(core) = core.as_mut() {
                        if removed && core.real_card {
                            let newent = self
                                .db
                                .spawn_card(&mut self.ents, &core.name, core.owner)
                                .unwrap();
                            match dest {
                                Zone::Exile
                                | Zone::Stack
                                | Zone::Command
                                | Zone::Battlefield
                                | Zone::Graveyard => {
                                    core.known.extend(self.turn_order.iter());
                                    //Public zone
                                    //Morphs **are** publicly known, just some attributes of face
                                    //down cards will not be known. I may need a second
                                    //structure for face down cards to track who knows what they are
                                    //For MDFC and TDFC cards this isn't an issue because
                                    //from one side, you know what the other side is.
                                    //For moving to the hand or library
                                    //it retains it's current knowledge set
                                    //Shuffling will destroy all knowledge of cards in the library
                                }
                                Zone::Hand => {
                                    core.known.insert(core.owner);
                                }
                                _ => {}
                            }
                            self.ents.insert_one(newent, core.clone()).unwrap();
                            let mut player = self.ents.get_mut::<Player>(core.owner).unwrap();
                            match dest {
                                Zone::Exile => {
                                    self.exile.insert(newent);
                                }
                                Zone::Command => {
                                    self.command.insert(newent);
                                }
                                Zone::Battlefield => {
                                    self.battlefield.insert(newent);
                                }
                                Zone::Hand => {
                                    player.hand.insert(newent);
                                }
                                //Handle inserting a distance from the top. Perhaps swap them afterwards?
                                Zone::Library => player.deck.push(newent),
                                Zone::Graveyard => player.graveyard.push(newent),
                                Zone::Stack => self.stack.push(newent),
                            }
                            results.push(EventResult::MoveZones {
                                oldent: ent,
                                newent: newent,
                                dest: dest,
                            });
                        }
                    }
                }
            }
        }
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
