use crate::ability::Ability;
use crate::card_entities::{CardEnt, EntType};
use crate::carddb::CardDB;
use crate::cost::{Cost, PaidCost};
use crate::ent_maps::EntMap;
use crate::entities::{CardId, ManaId, PlayerId, TargetId};
use crate::errors::MTGError;
use crate::event::{DiscardCause, Event, EventResult, TagEvent};
use crate::mana::{Color, Mana, ManaCostSymbol};
use crate::player::{AskReason, Player, PlayerCon};
use crate::spellabil::{Clause, ClauseEffect, KeywordAbility};
use anyhow::{bail, Result};
use async_recursion::async_recursion;
use enum_map::EnumMap;
use futures::future;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::Serialize;
use serde_derive::Serialize;
use serde_json;
use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use warp::ws::WebSocket;

use crate::actions::{Action, ActionFilter, CastingOption, StackActionOption};
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
    pub mana: EntMap<ManaId, Mana>,
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
            mana_pool: HashSet::new(),
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
            mana: EntMap::new(),
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
        panic!("restoring is a bug for now!");
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
            ManaCostSymbol::Generic => vec![Color::Colorless],
            ManaCostSymbol::Colorless => vec![Color::Colorless],
        };
        let mut ids = Vec::new();
        if let Some(pl) = self.players.get_mut(player) {
            for color in colors {
                let mana = Mana::new(color);
                let (id, _) = self.mana.insert(mana);
                pl.mana_pool.insert(id);
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
    #[must_use]
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
    pub async fn move_zones(&mut self, ent: CardId, origin: Zone, dest: Zone) -> Vec<CardId> {
        let moved = self
            .handle_event(Event::MoveZones { ent, origin, dest })
            .await;
        self.parse_zone_move(moved, dest)
    }
    fn parse_zone_move(&self, events: Vec<EventResult>, dest_zone: Zone) -> Vec<CardId> {
        let mut new_ents = Vec::new();
        for event in events {
            if let EventResult::MoveZones { oldent, newent, dest }=event
            && let Some(newent)=newent
            && dest==dest_zone{
                new_ents.push(newent);
            }
        }
        new_ents
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
                        self.backup();
                        let card = casting_option.source_card;
                        let stackobj = self
                            .move_zones(card, casting_option.zone, Zone::Stack)
                            .await;
                        if stackobj.len() != 1 {
                            self.restore();
                            continue;
                        }
                        let stack_opt = StackActionOption {
                            stack_ent: stackobj[0],
                            ability_source: None,
                            costs: casting_option.costs.clone(),
                            filter: ActionFilter::None,
                            keyword: None,
                            player,
                        };
                        if let Ok(_) = self.handle_cast(stack_opt).await {
                        } else {
                            self.restore();
                            continue;
                        }
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
                        let built = self.construct_activated_ability(player, *source, *index);
                        let (id, keyword) = if let Some(built) = built {
                            built
                        } else {
                            self.restore();
                            continue;
                        };
                        let costs = if let Some(card) = self.cards.get(id) {
                            card.costs.clone()
                        } else {
                            return false;
                        };
                        let castopt = StackActionOption {
                            costs,
                            filter: ActionFilter::None,
                            keyword,
                            player,
                            stack_ent: id,
                            ability_source: Some(*source),
                        };
                        if let Ok(_) = self.handle_cast(castopt).await {
                        } else {
                            self.restore();
                            continue;
                        }
                    }
                }
                return true;
            }
        }
    }
    //The spell has already been moved to the stack for this operation
    async fn handle_cast(&mut self, castopt: StackActionOption) -> Result<(), MTGError> {
        println!("Handling cast {:?}", castopt);
        let cost_paid = self.request_cost_payment(&castopt).await?;
        println!("cost paid {:?}", cost_paid);
        if self.is_mana_ability(castopt.stack_ent) {
            self.resolve(castopt.stack_ent).await;
        } else {
            //TODO handle rest of spellcasting
            let caster = castopt.player;
            let mut order = self.turn_order.clone();
            for _ in 0..order.len() {
                if order[0] == caster {
                    break;
                } else {
                    order.rotate_left(1);
                }
            }
            self.player_cycle_priority(order).await;
            self.resolve(castopt.stack_ent).await;
        }
        Ok(())
    }
    async fn allow_mana_abils(&mut self, player: PlayerId) {
        //TODO allow for activating mana sources while
        //paying for a mana cost, not just before a spell
    }
    //This function is a stub, it will
    //need to be expanded with real restrictions later
    fn can_spend_mana_on_action(&self, action: &StackActionOption, mana: &Mana) -> bool {
        if let Some(restriction) = &mana.restriction {
            restriction.approve(self, action)
        } else {
            true
        }
    }
    async fn request_mana_payment(
        &mut self,
        action: &StackActionOption,
        mut costs: Vec<ManaCostSymbol>,
    ) -> Result<Vec<PaidCost>, MTGError> {
        let player = action.player;
        self.allow_mana_abils(player).await;
        let mut mana_map: EnumMap<_, Vec<ManaId>> = EnumMap::default();
        if let Some(player) = self.players.get(player) {
            for &mana_id in player.mana_pool.iter() {
                if let Some(mana) = self.mana.get(mana_id) {
                    if self.can_spend_mana_on_action(action, mana) {
                        mana_map[mana.color].push(mana_id);
                    }
                }
            }
        }
        costs.sort();
        let mut spent_mana = Vec::new();
        'outer: for cost in costs {
            for color in cost.spendable_colors() {
                if let Some(mana) = mana_map[color].pop() {
                    spent_mana.push(mana);
                    continue 'outer;
                }
            }
            return Err(MTGError::CostNotPaid);
        }
        if let Some(player) = self.players.get_mut(player) {
            for mana in &spent_mana {
                player.mana_pool.remove(mana);
                //Dont delete mana from game so we can use it later
                //when cards need to know the mana spent on them
            }
            let res = spent_mana
                .iter()
                .map(|mana| PaidCost::PaidMana(*mana))
                .collect();
            Ok(res)
        } else {
            Err(MTGError::CostNotPaid)
        }
    }
    async fn request_cost_payment(
        &mut self,
        castopt: &StackActionOption,
    ) -> Result<Vec<PaidCost>, MTGError> {
        let mut mana_costs = Vec::new();
        let mut normal_costs = Vec::new();
        for &cost in &castopt.costs {
            if let Cost::Mana(color) = cost {
                mana_costs.push(color);
            } else {
                normal_costs.push(cost);
            }
        }
        let mut paid_costs = self.request_mana_payment(castopt, mana_costs).await?;
        for cost in normal_costs {
            let paid = match cost {
                Cost::Selftap => {
                    let tapped=if let Some(source_perm)=castopt.ability_source
                        && self.can_tap(source_perm){
                            paid_costs.push(PaidCost::Tapped(source_perm));
                            self.tap(source_perm).await
                    }else{
                        false
                    };
                    println!("tapped {:?}, {}", castopt.ability_source, tapped);
                    println!(
                        "{:?}",
                        castopt
                            .ability_source
                            .and_then(|source| self.cards.get(source))
                            .map(|card| card.tapped)
                    );
                    tapped
                }
                _ => {
                    todo!("Cost {:?} not implemented", cost)
                }
            };
            if !paid {
                return Err(MTGError::CostNotPaid);
            }
        }
        Ok(paid_costs)
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
        self.move_zones(id, Zone::Stack, dest).await;
    }
    async fn resolve_clause(&mut self, effect: ClauseEffect, id: CardId, controller: PlayerId) {
        match effect {
            ClauseEffect::AddMana(manas) => {
                for mana in manas {
                    self.add_mana(controller, mana).await;
                }
            }
            ClauseEffect::DrawCard => {
                self.draw(controller).await;
            }
        }
    }
    fn is_mana_ability(&self, id: CardId) -> bool {
        if let Some(card) = self.cards.get(id) {
            if !(card.ent_type == EntType::ActivatedAbility
                || card.ent_type == EntType::TriggeredAbility)
            {
                return false;
            }
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
    ) -> Option<(CardId, Option<KeywordAbility>)> {
        let card = self.cards.get(source)?;
        if index >= card.abilities.len() {
            return None;
        }
        let activated = match &card.abilities[index] {
            Ability::Activated(x) => Some(x),
            _ => None,
        }?;
        let mut abil = CardEnt::default();
        let keyword = activated.keyword;
        abil.ent_type = EntType::ActivatedAbility;
        abil.owner = player;
        abil.controller = Some(player);
        abil.costs = activated.costs.clone();
        abil.effect = activated.effect.clone();
        let (new_id, new_ent) = self.cards.insert(abil);
        Some((new_id, keyword))
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
            actions.extend(self.cast_actions(pl, player));
        }
        actions
    }
    pub fn cast_actions(&self, pl: &Player, player: PlayerId) -> Vec<Action> {
        let mut actions = Vec::new();
        for &card_id in pl.hand.iter() {
            if let Some(card) = self.cards.get(card_id) {
                if card.costs.len() > 0 && (card.types.instant || self.sorcery_speed(player)) {
                    actions.push(Action::Cast(CastingOption {
                        source_card: card_id,
                        costs: card.costs.clone(),
                        filter: ActionFilter::None,
                        zone: Zone::Hand,
                        player,
                    }));
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
                let abil = &card.abilities[i];
                let abil = match abil {
                    Ability::Activated(abil) => abil,
                    _ => continue,
                };

                let maybe_pay = self.maybe_can_pay(&abil.costs, player_id, card_id);
                if !maybe_pay {
                    continue;
                }
                actions.push(Action::ActivateAbility {
                    source: card_id,
                    index: i,
                })
            }
        }
        actions
    }
    pub fn maybe_can_pay(&self, costs: &Vec<Cost>, player_id: PlayerId, card_id: CardId) -> bool {
        if let Some(player) = self.players.get(player_id) {
            let mut available_mana: i64 = 0; //TODO make this take into account costs more accurately,
                                             //including handling colors of available mana, no just the quanitity
            for perm in self.players_permanents(player_id) {
                available_mana += self.max_mana_produce(perm);
            }
            available_mana += player.mana_pool.len() as i64;
            for cost in costs {
                let can_pay = match cost {
                    Cost::Selftap => self.battlefield.contains(&card_id) && self.can_tap(card_id),
                    Cost::Mana(mana) => {
                        if available_mana <= 0 {
                            false
                        } else {
                            available_mana -= 1;
                            true
                        }
                    }
                    _ => todo!(),
                };
                if !can_pay {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
    fn max_mana_produce(&self, ent: CardId) -> i64 {
        //TODO get more fine grained color support
        let mut mana_produce = 0;
        if let Some(ent) = self.cards.get(ent) {
            for ability in &ent.abilities {
                if let Ability::Activated(abil) = ability {
                    let mut abil_mana: i64 = 0;
                    for clause in &abil.effect {
                        if let Clause::Effect { effect } = clause {
                            if let ClauseEffect::AddMana(manas) = effect {
                                abil_mana += manas.len() as i64; //TODO handle replacement affacts adding mana
                            }
                        }
                    }
                    mana_produce = max(mana_produce, abil_mana);
                }
            }
        }
        return mana_produce;
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
            .filter_map(|&x| if x == player { None } else { Some(x) })
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
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
