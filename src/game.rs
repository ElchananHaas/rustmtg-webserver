use crate::ability::Ability;
use crate::card_entities::{CardEnt, EntType};
use crate::carddb::CardDB;
use crate::cost::Cost;
use crate::ent_maps::EntMap;
use crate::entities::{CardId, ManaId, PlayerId, TargetId};
use crate::event::{DiscardCause, Event, EventResult, TagEvent};
use crate::mana::{Color, Mana, ManaCostSymbol};
use crate::player::{AskReason, Player, PlayerCon};
use crate::spellabil::{Clause, ClauseEffect, KeywordAbility};
use anyhow::{bail, Result};
use async_recursion::async_recursion;
use futures::future;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::Serialize;
use serde_derive::Serialize;
use serde_json;
use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use warp::ws::WebSocket;

use self::actions::Action;
mod actions;
mod handle_event;
mod layers;

pub type Players = EntMap<PlayerId, Player>;
pub type Cards = EntMap<CardId, CardEnt>;
pub struct GameBuilder {
    players: Players,
    cards: Cards,
    turn_order: VecDeque<PlayerId>,
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
    pub turn_order: VecDeque<PlayerId>,
    pub extra_turns: VecDeque<PlayerId>,
    pub phases: VecDeque<Phase>,
    pub subphases: VecDeque<Subphase>,
    pub phase: Option<Phase>,
    pub subphase: Option<Subphase>,
    pub outcome: GameOutcome,
    pub lands_played_this_turn: u32,
    pub land_play_limit: u32,
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
            turn_order: VecDeque::new(),
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
        let (player_id, player) = self.players.insert(player);
        for cardname in card_names {
            let card: CardEnt = db.spawn_card(cardname, player_id);
            let (card_id, _card) = self.cards.insert(card);
            cards.push(card_id);
        }
        //Now that the deck has been constructed, set the players deck
        player.library = cards;
        self.turn_order.push_back(player_id);
        Ok(player_id)
    }
    pub fn build(self, db: &'static CardDB) -> Result<Game> {
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
            db,
            land_play_limit: 1,
            lands_played_this_turn: 0,
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
                self.handle_event(Event::Turn { extra: true }).await;
            } else {
                //Make sure the active player is updated properly if a player loses or leaves!
                let order_spot = self
                    .turn_order
                    .iter()
                    .position(|x| *x == self.active_player())
                    .unwrap();
                let new_spot = (order_spot + 1) % self.turn_order.len();
                let player = self.turn_order[new_spot];
                self.handle_event(Event::Turn { extra: false }).await;
            }
        }
        self.outcome
    }
    pub fn active_player(&self) -> PlayerId {
        self.turn_order[0]
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
        let mut b = None;
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
    pub async fn add_mana(&mut self, player: PlayerId, mana: ManaCostSymbol) -> Vec<ManaId> {
        let colors: Vec<Color> = match mana {
            ManaCostSymbol::Black => vec![Color::Black],
            ManaCostSymbol::Blue => vec![Color::Blue],
            ManaCostSymbol::Green => vec![Color::Green],
            ManaCostSymbol::Red => vec![Color::Red],
            ManaCostSymbol::White => vec![Color::White],
            ManaCostSymbol::Generic(x) => vec![Color::Colorless].repeat(x.try_into().unwrap()),
            ManaCostSymbol::Colorless => vec![Color::Colorless],
        };
        let mut ids = Vec::new();
        if let Some(pl) = self.players.get_mut(player) {
            for color in colors {
                let mana = Mana::new(color);
                let (id, _) = pl.mana_pool.insert(mana);
                ids.push(id);
            }
        }
        ids
    }
    pub fn players_creatures<'b>(&'b self, player: PlayerId) -> impl Iterator<Item = CardId> + 'b {
        self.all_creatures()
            .into_iter()
            .filter(move |&ent| self.get_controller(ent) == Some(player))
    }
    pub fn cards_and_zones(&self) -> Vec<(CardId, Zone)> {
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
    pub fn locate_zone(&self, id: CardId) -> Option<Zone> {
        for (ent_id, zone) in self.cards_and_zones() {
            if ent_id == id {
                return Some(zone);
            }
        }
        None
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
        self.player_cycle_priority(self.turn_order.clone()).await;
    }
    #[async_recursion]
    pub async fn player_cycle_priority(&mut self, mut players: VecDeque<PlayerId>) {
        self.place_abilities().await;
        for _ in 0..players.len() {
            self.send_state().await;
            let act_taken = self.grant_priority(&players).await;
            if act_taken {
                self.player_cycle_priority(players.clone()).await;
            }
            players.rotate_left(1);
        }
    }
    pub async fn grant_priority(&mut self, players: &VecDeque<PlayerId>) -> bool {
        let player = players[0];
        self.layers();
        loop {
            let actions = self.compute_actions(player);
            let mut choice = Vec::new();
            if let Some(pl) = self.players.get(player) {
                choice = pl.ask_user_selectn(&actions, 0, 1, AskReason::Action).await;
            }
            if choice.len() == 0 {
                return false;
            } else {
                let action = &actions[choice[0]];
                match action {
                    Action::Cast(casting_option) => {
                        todo!()
                    }
                    Action::PlayLand(card) => {
                        self.handle_event(Event::PlayLand {
                            player,
                            land: *card,
                        })
                        .await;
                    }
                    Action::ActivateAbility { source, index } => {
                        self.backup();
                        let id = self.construct_activated_ability(player, *source, *index);
                        let id = if let Some(id) = id {
                            id
                        } else {
                            self.restore();
                            continue;
                        };
                        let cost_paid = self.request_cost_payment(id, *source).await;
                        if !cost_paid {
                            self.restore();
                            continue;
                        }
                        if self.is_mana_ability(id) {
                            self.resolve(id).await;
                        } else {
                            //TODO handle rest of spellcasting
                            todo!();
                        }
                    }
                }
                return true;
            }
        }
    }
    async fn request_cost_payment(&mut self, id: CardId, source: CardId) -> bool {
        println!("on to cost payment");
        let costs = if let Some(card) = self.cards.get(id) {
            card.costs.clone()
        } else {
            return false;
        };
        for cost in costs {
            match cost {
                Cost::Selftap => {
                    if !self.can_tap(source) {
                        return false;
                    }
                    self.tap(source).await;
                }
                _ => {
                    todo!()
                }
            }
        }
        println!("paid costs");
        true
    }
    async fn resolve(&mut self, id: CardId) {
        let effects;
        let controller;
        let types;
        if let Some(ent) = self.cards.get(id) {
            effects = ent.effect.clone();
            controller = ent.get_controller();
            types = ent.types.clone();
        } else {
            return;
        }
        for effect in effects {
            match effect {
                Clause::Effect { effect } => {
                    self.resolve_clause(effect, id, controller).await;
                }
                _ => todo!(),
            }
        }
        let dest = if types.instant || types.sorcery {
            Zone::Graveyard
        } else {
            Zone::Battlefield
        };
        self.handle_event(Event::MoveZones {
            ent: id,
            origin: Zone::Stack,
            dest,
        })
        .await;
    }
    async fn resolve_clause(&mut self, effect: ClauseEffect, id: CardId, controller: PlayerId) {
        match effect {
            ClauseEffect::AddMana(manas) => {
                for mana in manas {
                    self.add_mana(controller, mana).await;
                }
            }
        }
    }
    fn is_mana_ability(&self, id: CardId) -> bool {
        if let Some(card) = self.cards.get(id) {
            for cost in &card.costs {
                //Check for loyalty abilities when implemented
            }
            let mut mana_abil = false;
            for clause in &card.effect {
                match clause {
                    Clause::Target {
                        targets: _,
                        effect: _,
                    } => {
                        return false;
                    }
                    Clause::Effect { effect } => match effect {
                        ClauseEffect::AddMana(_) => {
                            mana_abil = true;
                        }
                        _ => (),
                    },
                }
            }
            mana_abil
        } else {
            false
        }
    }
    fn construct_activated_ability(
        &mut self,
        player: PlayerId,
        source: CardId,
        index: usize,
    ) -> Option<CardId> {
        let card = self.cards.get(source)?;
        if index >= card.abilities.len() {
            return None;
        }
        let activated = match &card.abilities[index] {
            Ability::Activated(x) => Some(x),
            _ => None,
        }?;
        let mut abil = CardEnt::default();
        abil.ent_type = EntType::ActivatedAbility;
        abil.owner = player;
        abil.controller = Some(player);
        abil.costs = activated.costs.clone();
        abil.effect = activated.effect.clone();
        let (new_id, new_ent) = self.cards.insert(abil);
        Some(new_id)
    }
    pub fn compute_actions(&self, player: PlayerId) -> Vec<Action> {
        let mut actions = Vec::new();
        if let Some(pl) = self.players.get(player) {
            for &card_id in &pl.hand {
                if let Some(card) = self.cards.get(card_id) {
                    actions.extend(self.play_land_actions(player, pl, card_id, card));
                }
            }
            for (card_id, zone) in self.cards_and_zones() {
                if let Some(card) = self.cards.get(card_id) {
                    actions.extend(self.ability_actions(player, pl, card_id, card, zone));
                }
            }
        }
        actions
    }
    fn sorcery_speed(&self, player_id: PlayerId) -> bool {
        player_id == self.active_player()
            && self.stack.is_empty()
            && (self.phase == Some(Phase::FirstMain) || self.phase == Some(Phase::SecondMain))
    }
    fn play_land_actions(
        &self,
        player_id: PlayerId,
        player: &Player,
        card_id: CardId,
        card: &CardEnt,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let play_sorcery = self.sorcery_speed(player_id);
        if card.types.land && play_sorcery && self.land_play_limit > self.lands_played_this_turn {
            actions.push(Action::PlayLand(card_id));
        }
        actions
    }
    fn ability_actions(
        &self,
        player_id: PlayerId,
        player: &Player,
        card_id: CardId,
        card: &CardEnt,
        zone: Zone,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let controller = card.controller.unwrap_or(card.owner);
        for i in 0..card.abilities.len() {
            if zone == Zone::Battlefield && controller == player_id {
                let abil=&card.abilities[i];
                let abil=match abil{
                    Ability::Activated(abil)=>abil,
                    _=>continue
                };
                for cost in &abil.costs{
                    if !self.maybe_can_pay(card_id,card,cost,zone){
                        continue;
                    }
                }
                actions.push(Action::ActivateAbility {
                    source: card_id,
                    index: i,
                })
            }
        }
        actions
    }
    pub fn maybe_can_pay(&self,card_id:CardId,card:&CardEnt,cost:&Cost,zone:Zone)->bool{
        match cost{
            Cost::Selftap=>{
                zone==Zone::Battlefield && self.can_tap(card_id)
            }
            _=>true
        }
    }
    //Places abilities on the stack
    pub async fn place_abilities(&mut self) {
        //TODO!
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
