use crate::ability::Ability;
use crate::card_entities::{CardEnt, PT};
use crate::carddb::CardDB;
use crate::components::Subtype;
use crate::entities::{CardId, ManaId, PlayerId, TargetId};
use crate::event::{DiscardCause, Event, EventResult, TagEvent};
use crate::mana::{Color, Mana, ManaCostSymbol};
use crate::player::{Player, PlayerCon};
use crate::spellabil::KeywordAbility;
use crate::AppendableMap::EntMap;
use anyhow::{bail, Result};
use futures::future;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_derive::Serialize;
use serde_json;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use warp::ws::WebSocket;
mod handle_event;
mod layers;

pub type Players = EntMap<PlayerId, Player>;
pub type Cards = EntMap<CardId, CardEnt>;
pub struct GameBuilder {
    players: Players,
    cards: Cards,
    turn_order: Vec<PlayerId>,
    active_player: Option<PlayerId>,
}
//Implement debug trait!
#[derive(Serialize, Clone)]
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
    backup: Option<Box<Game>>,
    #[serde(skip_serializing)]
    rng: rand::rngs::StdRng, //Store the RNG to allow for deterministic replay
                             //if I choose to implement it
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
        card_names: &Vec<&'static str>,
        player_con: WebSocket,
    ) -> Result<PlayerId> {
        let mut cards = Vec::new();
        let player = Player {
            name: name.to_owned(),
            hand: HashSet::new(),
            life: 20,
            mana_pool: EntMap::new(),
            graveyard: Vec::new(),
            library: Vec::new(),
            max_handsize: 7,
            player_con: PlayerCon::new(player_con),
        };
        let (player_id,player) = self.players.insert(player);
        for cardname in card_names {
            let card: CardEnt = db.spawn_card( &cardname, player_id);
            let (card_id,card)=self.cards.insert(card);
            cards.push(card_id);
        }
        //Now that the deck has been constructed, set the players deck
        player.library = cards;
        self.turn_order.push(player_id);
        if self.active_player.is_none() {
            self.active_player = Some(player_id);
        }
        Ok(player_id)
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

//This is a helper struct for game serialization because
//the function takes a mutable context,
//so serialize needs to be implemented on a different struct

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
        let mut card_views = HashMap::new();
        for (card_id, card_ref) in self.cards.view() {
            if card_ref.known_to.contains(&player) {
                card_views.insert(card_id, card_ref);
            }
        }
        let mut player_views = HashMap::new();
        for (player_id, player_ref) in self.players.view() {
            let view = player_ref.view(&self.cards, player);
            player_views.insert(player_id, view);
        }
        let mut buffer = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut json_serial = serde_json::Serializer::new(cursor);
            let added_context = ("GameState", player, card_views, player_views, self);
            added_context.serialize(&mut json_serial)?;
        }
        if let Some(pl) = self.players.get(player) {
            pl.send_state(buffer).await?;
        }
        Ok(())
    }

    //backs the game up in case a spell casting or attacker/blocker
    //declaration fails. Only backs up what is needed
    pub fn backup(&mut self) {
        if self.backup.is_some() {
            self.backup = None;
        }
        self.backup = Some(Box::new(self.clone()));
    }
    pub fn restore(&mut self) {
        let mut b=None;
        std::mem::swap(&mut b, &mut self.backup);
        *self = *b.unwrap();
    }
    //Taps an entity, returns if it was sucsessfully tapped
    pub async fn tap(&mut self, ent: CardId) -> bool {
        self.handle_event(Event::Tap { ent })
            .await
            .contains(&EventResult::Tap(ent))
    }
    //Taps an entity, returns if it was sucsessfully tapped
    pub async fn untap(&mut self, ent: CardId) -> bool {
        self.handle_event(Event::Untap { ent })
            .await
            .contains(&EventResult::Untap(ent))
    }
    //draws a card, returns the entities drawn
    pub async fn draw(&mut self, player: PlayerId) -> Vec<CardId> {
        let res = self.handle_event(Event::Draw { player }).await;
        let drawn = Vec::new();
        drawn
        //TODO figure out which cards were drawn!
    }
    pub fn shuffle(&mut self, player: PlayerId) {
        if let Some(pl) = self.players.get_mut(player) {
            pl.library.shuffle(&mut self.rng);
        }
    }
    //discard cards, returns discarded cards
    pub async fn discard(
        &mut self,
        player: PlayerId,
        card: CardId,
        cause: DiscardCause,
    ) -> Vec<CardId> {
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
    pub async fn add_mana(&mut self, player: PlayerId, mana: ManaCostSymbol) -> Option<ManaId> {
        let color: Color = match mana {
            ManaCostSymbol::Black => Color::Black,
            ManaCostSymbol::Blue => Color::Blue,
            ManaCostSymbol::Green => Color::Green,
            ManaCostSymbol::Red => Color::Red,
            ManaCostSymbol::White => Color::White,
            ManaCostSymbol::Generic => Color::Colorless,
            ManaCostSymbol::Colorless => Color::Colorless,
        };
        let pl = self.players.get_mut(player)?;
        let mana = Mana::new(color);
        let (id,_) = pl.mana_pool.insert(mana);
        Some(id)
    }
    pub fn players_creatures<'b>(&'b self, player: PlayerId) -> impl Iterator<Item = CardId> + 'b {
        self.all_creatures()
            .into_iter()
            .filter(move |&ent| self.get_controller(ent) == Some(player))
    }
    pub fn ents_and_zones(&self) -> Vec<(CardId, Zone)> {
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
            if let Some(player) = self.players.get(player_id) {
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
    pub fn players_permanents<'b>(&'b self, player: PlayerId) -> impl Iterator<Item = CardId> + 'b {
        self.battlefield
            .clone()
            .into_iter()
            .filter(move |&ent| self.get_controller(ent) == Some(player))
    }
    pub fn all_creatures<'b>(&'b self) -> impl Iterator<Item = CardId> + 'b {
        self.battlefield.clone().into_iter().filter(move |&ent| {
            self.cards
                .get(ent)
                .filter(|&card| card.types.creature)
                .is_some()
        })
    }
    //Can this creature tap to be declared an attacker or to activate an ability?
    //Doesn't include prevention effects, just if it can tap w/o them
    pub fn can_tap(&self, ent: CardId) -> bool {
        if let Some(card) = self.cards.get(ent) {
            if card.tapped {
                return false;
            }
            !card.types.creature
                || card.has_keyword(KeywordAbility::Haste)
                || !card.summoning_sickness
        } else {
            false
        }
    }
    //takes in a card or permanent, returns it's controller or owner if the controller
    //is unavailable
    pub fn get_controller(&self, ent: CardId) -> Option<PlayerId> {
        self.cards
            .get(ent)
            .and_then(|card| card.controller.or(Some(card.owner)))
    }
    pub async fn cycle_priority(&mut self) {
        self.place_abilities().await;
        for player in self.turn_order.clone() {
            self.grant_priority(player).await;
        }
    }
    pub async fn grant_priority(&mut self, player: PlayerId) {
        self.layers();
        //TODO actually grant priority
    }
    //Places abilities on the stack
    pub async fn place_abilities(&mut self) {
        //TODO make this do something!
    }
    pub fn attack_targets(&self, player: PlayerId) -> Vec<TargetId> {
        self.opponents(player)
            .iter()
            .map(|pl| TargetId::Player(*pl))
            .collect::<Vec<_>>()
    }

    pub fn opponents(&self, player: PlayerId) -> Vec<PlayerId> {
        self.turn_order
            .iter()
            .filter_map(|x| if *x == player { Some(*x) } else { None })
            .collect()
    }
    //Checks if this attacking arragment is legal.
    //Does nothing for now, will need to implement legality
    //checking before I can make any progress on that
    pub fn attackers_legal(&self, attackers: &Vec<CardId>, targets: &Vec<TargetId>) -> bool {
        true
    }
    pub fn blocks_legal(&self, blockers: &Vec<CardId>, blocked: &Vec<Vec<CardId>>) -> bool {
        true
    }
    pub fn remaining_lethal(&self, ent: CardId) -> Option<i64> {
        self.cards
            .get(ent)
            .and_then(|card| card.pt.map(|pt| max(pt.toughness - card.damaged, 0)))
    }
    pub fn add_ability(&mut self, ent: CardId, ability: Ability) {
        //Assume the builder has already added a vector of abilities
        if let Some(ent) = self.cards.get_mut(ent) {
            ent.abilities.push(ability);
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
