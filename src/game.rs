use crate::ability::Ability;
use crate::actions::{Action, ActionFilter, CastingOption, StackActionOption};
use crate::card_entities::{CardEnt, EntType};
use crate::carddb::CardDB;
use crate::client_message::{Ask, AskSelectN};
use crate::cost::{Cost, PaidCost};
use crate::ent_maps::EntMap;
use crate::entities::{CardId, ManaId, PlayerId, TargetId, MIN_CARDID};
use crate::errors::MTGError;
use crate::event::{DiscardCause, Event, EventResult, TagEvent};
use crate::hashset_obj::HashSetObj;
use crate::mana::{Color, Mana, ManaCostSymbol};
use crate::player::{Player, PlayerCon};
use crate::spellabil::{Affected, Clause, ClauseEffect, KeywordAbility};
use anyhow::{bail, Result};
use async_recursion::async_recursion;
use enum_map::EnumMap;
use futures::future;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use schemars::JsonSchema;
use serde::Serialize;
use std::cmp::max;
use std::collections::{HashMap, VecDeque};
use std::num::ParseFloatError;

pub mod build_game;
mod compute_actions;
mod event_generators;
mod handle_event;
mod layers_state_actions;
mod resolve;
mod serialize_game;

pub type Players = EntMap<PlayerId, Player>;
pub type Cards = EntMap<CardId, CardEnt>;

#[derive(Serialize, Clone, JsonSchema)]
pub struct Game {
    #[serde(skip)]
    pub players: Players,
    #[serde(skip)]
    pub cards: Cards,
    pub mana: EntMap<ManaId, Mana>,
    pub battlefield: HashSetObj<CardId>,
    pub exile: HashSetObj<CardId>,
    pub command: HashSetObj<CardId>,
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
    pub priority: PlayerId,
    pub active_player: PlayerId,
    #[serde(skip)]
    db: &'static CardDB,
    #[serde(skip)]
    backup: Option<Box<Game>>,
    #[serde(skip)]
    rng: rand::rngs::StdRng, //Store the RNG to allow for deterministic replay
                             //if I choose to implement it
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, JsonSchema)]
pub enum GameOutcome {
    Ongoing,
    Tie,
    Winner(PlayerId),
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
                self.turn_order.rotate_left(1);
                self.handle_event(Event::Turn {
                    player: self.turn_order[0],
                    extra: false,
                })
                .await;
            }
        }
        self.outcome
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
        println!("restoring!");
        let mut b = None;
        std::mem::swap(&mut b, &mut self.backup);
        *self = *b.unwrap();
        self.backup()
    }

    pub fn shuffle(&mut self, player: PlayerId) {
        if let Some(pl) = self.players.get_mut(player) {
            pl.library.shuffle(&mut self.rng);
        }
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
            !card.types.creature || card.has_keyword(KeywordAbility::Haste) || !card.etb_this_cycle
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
        self.player_cycle_priority(self.turn_order_from_player(self.active_player))
            .await;
    }
    #[async_recursion]
    #[must_use]
    pub async fn player_cycle_priority(&mut self, mut players: VecDeque<PlayerId>) {
        let mut pass_count = 0;
        while pass_count < players.len() {
            self.layers_state_actions().await;
            self.priority = players[0];
            self.send_state().await;
            let act_taken = self.grant_priority(&players).await;
            match act_taken {
                ActionPriorityType::Pass => {
                    pass_count += 1;
                    players.rotate_left(1);
                }
                ActionPriorityType::ManaAbilOrSpecialAction => {
                    //Do nothing, the player gains priority again.
                    //This happens for mana abilities and special abiltiies
                }
                ActionPriorityType::Action => {
                    //Do nothing, but there was a cycling of priority
                }
            }
        }
    }

    pub async fn grant_priority(&mut self, players: &VecDeque<PlayerId>) -> ActionPriorityType {
        let player = players[0];
        loop {
            let actions = self.compute_actions(player);
            let mut choice = Vec::new();
            if let Some(pl) = self.players.get(player) {
                let select = AskSelectN {
                    ents: actions.clone(),
                    min: 0,
                    max: 1,
                };
                choice = pl
                    .ask_user_selectn(&Ask::Action(select.clone()), &select)
                    .await;
            }
            if choice.len() == 0 {
                return ActionPriorityType::Pass;
            } else {
                let action = &actions[choice[0]];
                let _: ! = match action {
                    Action::Cast(casting_option) => {
                        self.backup();
                        let card = casting_option.source_card;
                        let stackobjs = self
                            .move_zones(card, casting_option.zone, Zone::Stack)
                            .await;
                        let stack_ent;
                        if stackobjs.len() == 1 {
                            if let EventResult::MoveZones {
                                oldent: _,
                                newent: Some(newent),
                                source: _,
                                dest: _,
                            } = stackobjs[0]
                            {
                                stack_ent = newent
                            } else {
                                self.restore();
                                continue;
                            }
                        } else {
                            self.restore();
                            continue;
                        }
                        let stack_opt = StackActionOption {
                            stack_ent,
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
                        return ActionPriorityType::Action;
                    }
                    Action::PlayLand(card) => {
                        self.handle_event(Event::PlayLand {
                            player,
                            land: *card,
                        })
                        .await;
                        return ActionPriorityType::ManaAbilOrSpecialAction;
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
                            return ActionPriorityType::Pass;
                        };
                        let mana_abil = self.is_mana_ability(id);
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
                        if mana_abil {
                            return ActionPriorityType::ManaAbilOrSpecialAction;
                        } else {
                            return ActionPriorityType::Action;
                        }
                    }
                };
            }
        }
    }
    fn turn_order_from_player(&self, player: PlayerId) -> VecDeque<PlayerId> {
        let mut order = self.turn_order.clone();
        for _ in 0..order.len() {
            if order[0] == player {
                break;
            } else {
                order.rotate_left(1);
            }
        }
        order
    }
    async fn select_targets(&mut self, castopt: &StackActionOption) -> Result<(), MTGError> {
        let cards_and_zones = self.cards_and_zones();
        let mut selected_targets = Vec::new();
        if let Some(card) = self.cards.get(castopt.stack_ent) {
            for clause in &card.effect {
                let mut selected_target = None;
                if let Affected::Target { target } = clause.affected {
                    if let Some(pl) = self.players.get(castopt.player) {
                        let mut valid = Vec::new();
                        for &(card, zone) in &cards_and_zones {
                            if clause
                                .constraints
                                .iter()
                                .all(|x| x.passes_constraint(&self, card.into()))
                            {
                                valid.push(TargetId::Card(card))
                            }
                        }
                        let ask = AskSelectN {
                            ents: valid.clone(),
                            min: 1,
                            max: 1,
                        };
                        let choice = pl.ask_user_selectn(&Ask::Target(ask.clone()), &ask).await;
                        selected_target = Some(valid[choice[0]]);
                    } else {
                        return Err(MTGError::PlayerDoesntExist);
                    }
                }
                selected_targets.push(selected_target);
            }
        } else {
            return Err(MTGError::CastNonExistentSpell);
        }
        let card = self
            .cards
            .get_mut(castopt.stack_ent)
            .expect("card was checked to exist");
        for (i, clause) in card.effect.iter_mut().enumerate() {
            if let Affected::Target { target } = clause.affected {
                let target = selected_targets[i];
                if let Some(t) = target {
                    clause.affected = Affected::Target { target: Some(t) };
                } else {
                    return Err(MTGError::TargetNotChosen);
                }
            }
        }
        Ok(())
    }
    //The spell has already been moved to the stack for this operation
    async fn handle_cast(&mut self, castopt: StackActionOption) -> Result<(), MTGError> {
        println!("Handling cast {:?}", castopt);
        if !castopt.filter.check() {
            return Err(MTGError::CantCast);
        }
        self.select_targets(&castopt).await?;
        let cost_paid = self.request_cost_payment(&castopt).await?;
        println!("cost paid {:?}", cost_paid);
        if self.is_mana_ability(castopt.stack_ent) {
            self.resolve(castopt.stack_ent).await;
        } else {
            //TODO handle rest of spellcasting
            let caster = castopt.player;
            let order = self.turn_order_from_player(caster);
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
    fn has_keyword(&self, id: CardId, keyword: KeywordAbility) -> bool {
        if let Some(card) = self.cards.get(id) {
            return card.has_keyword(keyword);
        }
        false
    }
    fn is_mana_ability(&self, id: CardId) -> bool {
        if let Some(card) = self.cards.get(id) {
            if !(card.ent_type == EntType::ActivatedAbility
                || card.ent_type == EntType::TriggeredAbility)
            {
                return false;
            }
            //TODO Check for loyalty abilities when implemented
            let mut mana_abil = false;
            let mut other_effect = false;
            for clause in &card.effect {
                match clause.effect {
                    ClauseEffect::AddMana(_) => {
                        mana_abil = true;
                    }
                    _ => {
                        other_effect = true;
                    }
                }
            }
            mana_abil & !other_effect
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
        let (new_id, _new_ent) = self.cards.insert(abil);
        Some((new_id, keyword))
    }
    fn sorcery_speed(&self, player_id: PlayerId) -> bool {
        player_id == self.active_player
            && self.stack.is_empty()
            && (self.phase == Some(Phase::FirstMain) || self.phase == Some(Phase::SecondMain))
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
                    Cost::Mana(_mana) => {
                        if available_mana <= 0 {
                            false
                        } else {
                            available_mana -= 1;
                            true
                        }
                    }
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
                        if let ClauseEffect::AddMana(mana) = &clause.effect {
                            abil_mana += mana.len() as i64;
                        }
                    }
                    mana_produce = max(mana_produce, abil_mana);
                }
            }
        }
        return mana_produce;
    }
    pub fn opponents(&self, player: PlayerId) -> Vec<PlayerId> {
        self.turn_order
            .iter()
            .filter_map(|&x| if x == player { None } else { Some(x) })
            .collect()
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

pub enum ActionPriorityType {
    Pass,
    ManaAbilOrSpecialAction,
    Action,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, JsonSchema)]
pub enum Phase {
    Begin,
    FirstMain,
    Combat,
    SecondMain,
    Ending,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, JsonSchema)]
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, JsonSchema)]
pub enum Zone {
    Hand,
    Library,
    Exile,
    Battlefield,
    Graveyard,
    Command,
    Stack,
}
