use crate::actions::{Action, ActionFilter, CastingOption, StackActionOption};
use crate::client_message::{Ask, AskSelectN};
use crate::ent_maps::EntMap;
use crate::errors::MTGError;
use crate::event::{Event, EventResult, TagEvent};
use crate::player::{Player, PlayerCon};
use crate::CARDDB;
use anyhow::{bail, Result};
use async_recursion::async_recursion;
use carddb::carddb::CardDB;
use common::ability::{Ability, StaticAbilityEffect};
use common::card_entities::{CardEnt, EntType};
use common::cardtypes::Subtype;
use common::cost::{Cost, PaidCost};
use common::entities::{CardId, ManaId, PlayerId, TargetId, MIN_CARDID};
use common::hashset_obj::HashSetObj;
use common::mana::{Color, Mana, ManaCostSymbol};
use common::spellabil::{Affected, Clause, ClauseEffect, Constraint, Continuous, KeywordAbility};
use common::zones::Zone;
use enum_map::EnumMap;
use futures::future;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::{HashMap, VecDeque};

pub mod build_game;
mod compute_actions;
mod event_generators;
mod handle_event;
mod layers_state_actions;
mod resolve;
mod serialize_game;

pub type Players = EntMap<PlayerId, Player>;
pub type Cards = EntMap<CardId, CardEnt>;

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
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
    pub cont_effects: Vec<Continuous>, //Holds continuous effects
    //that are perpetual or time-driven
    #[serde(skip, default = "get_carddb")]
    #[allow(dead_code)]
    db: &'static CardDB,
    #[serde(skip)]
    backup: Option<Box<Game>>,
    #[serde(skip, default = "rng_from_entropy")]
    rng: rand::rngs::StdRng, //Store the RNG to allow for deterministic replay
                             //if I choose to implement it
}

fn get_carddb() -> &'static CardDB {
    CARDDB.get_or_init(|| CardDB::new())
}

fn rng_from_entropy() -> rand::rngs::StdRng {
    rand::rngs::StdRng::from_entropy()
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
    pub fn all_cards(&self) -> Vec<CardId> {
        let mut res = Vec::new();
        res.extend(self.battlefield.iter().cloned());
        res.extend(self.stack.iter().cloned());
        res.extend(self.exile.iter().cloned());
        res.extend(self.command.iter().cloned());
        for player_id in self.turn_order.clone() {
            if let Some(player) = self.players.get(player_id) {
                res.extend(player.hand.iter().cloned());
                res.extend(player.graveyard.iter().cloned());
                res.extend(player.library.iter().cloned())
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
                .filter(|&card| card.types.is_creature())
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
            !card.types.is_creature()
                || card.has_keyword(KeywordAbility::Haste)
                || !card.etb_this_cycle
        } else {
            false
        }
    }
    //takes in a card or permanent, returns it's controller or owner if the controller
    //is unavailable
    pub fn get_controller(&self, ent: CardId) -> Option<PlayerId> {
        self.cards.get(ent).map(|card| card.get_controller())
    }
    pub async fn cycle_priority(&mut self) {
        loop {
            self.player_cycle_priority(self.turn_order_from_player(self.active_player))
                .await;
            if self.stack.len() == 0 {
                return;
            } else {
                self.resolve(self.stack[self.stack.len() - 1]).await;
                self.stack.pop();
            }
        }
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
            let mut choice = HashSetObj::new();
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
                let action = &actions[choice.into_iter().next().unwrap()];
                let _: ! = match action {
                    Action::Cast(casting_option) => {
                        self.backup();
                        let card = casting_option.source_card;
                        let stackobjs = self
                            .move_zones(vec![card], casting_option.zone, Zone::Stack)
                            .await;
                        let stack_ent;
                        if stackobjs.len() == 1 {
                            if let EventResult::MoveZones(events) = &stackobjs[0]
                            && events.len()==1
                            && let Some(newent)=events[0].newent
                            {
                                stack_ent = newent;
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
    pub fn passes_constraint(
        &self,
        constraint: &Constraint,
        source: CardId,
        target: TargetId,
    ) -> bool {
        match constraint{
            Constraint::HasCounter(counter) => {
                if let TargetId::Card(card)=target
                && let Some(ent)=self.cards.get(card){
                    ent.counters.contains(counter)
                }else{
                    false
                } 
            }
            Constraint::ControlWith(constraints,num )=>{
                if let Some(ent)=self.cards.get(source){
                    let mut count=0;
                    for card in self.players_permanents(ent.get_controller()){
                        if constraints.iter().all(|r| self.passes_constraint(r, card,card.into())){
                            count+=1
                        }
                    }
                    count>=*num
                }else{
                    false
                } 
            }
            Constraint::And(constraints) => {             
                for c in constraints{
                    if !self.passes_constraint(c, source,target){
                        return false
                    }
                }
                true
            }
            Constraint::Multicolored => {
                if let TargetId::Card(card)=target
                && let Some(ent)=self.cards.get(card){
                    ent.colors.len()>=2
                }else{
                    false
                } 
            }
            Constraint::Subtype(subtype)=>{
                if let TargetId::Card(card)=target
                && let Some(ent)=self.cards.get(card){
                    ent.subtypes.contains(subtype)
                }else{
                    false
                }
            }
            Constraint::IsTapped => {
                if let TargetId::Card(card)=target
                && let Some(ent)=self.cards.get(card){
                    ent.tapped
                }else{
                    false
                }
            },
            Constraint::CardType(t) => {
                if let TargetId::Card(card)=target
                && let Some(ent)=self.cards.get(card){
                    ent.types.get(t)
                }else{
                    false
                }                        
            },
            Constraint::Or(constraints) => {
                for c in constraints{
                    if self.passes_constraint(c, source,target){
                        return true
                    }
                }
                false
            }
            Constraint::IsCardname =>{
                if let TargetId::Card(cardid)=target{
                    return self.stack_ent_source(source)==cardid;
                }else{
                    true
                }
            },
            Constraint::Other =>{
                if let TargetId::Card(cardid)=target{
                    return !(self.stack_ent_source(source)==cardid);
                }else{
                    true
                }
            }
            Constraint::YouControl=>{
                if let Some(source)=self.cards.get(source)
                && let TargetId::Card(c)=target
                && let Some(target)=self.cards.get(c){
                    source.get_controller()==target.get_controller()
                }else{
                    false
                }
            }
            Constraint::HasKeyword(keyword)=>{
                if let TargetId::Card(card)=target
                && let Some(card)=self.cards.get(card){
                    card.has_keyword(*keyword)
                }else{
                    false
                }
            }
        }
    }
    async fn select_targets(
        &mut self,
        player: PlayerId,
        stack_ent: CardId,
    ) -> Result<(), MTGError> {
        let mut selected = Vec::new();
        if let Some(card) = self.cards.get(stack_ent) {
            for x in &card.effect {
                selected.push(self.clause_select_targets(player, stack_ent, x).await?);
            }
        }
        if let Some(card) = self.cards.get_mut(stack_ent) {
            card.effect = selected;
        }
        Ok(())
    }
    async fn clause_select_targets(
        &self,
        player: PlayerId,
        stack_ent: CardId,
        clause: &Clause,
    ) -> Result<Clause, MTGError> {
        let mut clause = clause.clone();
        return Ok(match &clause.affected {
            Affected::Cardname
            | Affected::Controller
            | Affected::ManuallySet(_)
            | Affected::All => clause,
            Affected::Target(_) => {
                if let Some(pl) = self.players.get(player) {
                    let valid = self.valid_targets(&clause, stack_ent);
                    if valid.len() == 0 {
                        return Err(MTGError::NoValidTargets);
                    }
                    let ask = AskSelectN {
                        ents: valid.clone(),
                        min: 1,
                        max: 1,
                    };
                    let choice = pl.ask_user_selectn(&Ask::Target(ask.clone()), &ask).await;
                    let target = valid[choice.into_iter().next().unwrap()];
                    clause.affected = Affected::Target(Some(target));
                    return Ok(clause);
                } else {
                    return Err(MTGError::PlayerDoesntExist);
                }
            }
            Affected::UpToXTarget(n, _) => {
                println!("selectng up to x targets");
                if let Some(pl) = self.players.get(player) {
                    let valid = self.valid_targets(&clause, stack_ent);
                    let ask = AskSelectN {
                        ents: valid.clone(),
                        min: 0,
                        max: *n,
                    };
                    let choice = pl.ask_user_selectn(&Ask::Target(ask.clone()), &ask).await;
                    let choices = choice.into_iter().map(|i| valid[i]).collect();
                    clause.affected = Affected::UpToXTarget(*n, choices);
                    return Ok(clause);
                } else {
                    return Err(MTGError::PlayerDoesntExist);
                }
            }
        });
    }
    pub fn valid_targets(&self, clause: &Clause, source: CardId) -> Vec<TargetId> {
        let mut valid = Vec::new();
        for &(card, zone) in &self.cards_and_zones() {
            if self.is_valid_target(&clause, source, card.into(), zone) {
                valid.push(TargetId::Card(card));
            }
        }
        valid
    }
    pub fn is_valid_target(
        &self,
        clause: &Clause,
        source: CardId,
        target: TargetId,
        _zone: Zone,
    ) -> bool {
        let source = self.stack_ent_source(source);
        if !clause
            .constraints
            .iter()
            .all(|x| self.passes_constraint(x, source, target))
        {
            return false;
        }
        if self.has_protection_from(source, target) {
            return false;
        }
        true
    }
    //The spell has already been moved to the stack for this operation
    async fn handle_cast(&mut self, castopt: StackActionOption) -> Result<(), MTGError> {
        println!("Handling cast {:?}", castopt);
        if !castopt.filter.check() {
            return Err(MTGError::CantCast);
        }
        self.select_targets(castopt.player, castopt.stack_ent)
            .await?;
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
            //TODO check if the actual restuction is met
            match restriction {
                _ => true,
            }
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
                    let tapped=
                    if let Some(card)=self.cards.get(castopt.stack_ent)
                    && let Some(source_perm)=card.source_of_ability
                        && self.can_tap(source_perm){
                            paid_costs.push(PaidCost::Tapped(source_perm));
                            self.tap(source_perm).await
                    }else{
                        false
                    };
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
        abil.set_controller(Some(player));
        abil.costs = activated.costs.clone();
        abil.effect = activated.effect.clone();
        abil.source_of_ability = Some(source);
        let (new_id, _new_ent) = self.cards.insert(abil);
        Some((new_id, keyword))
    }
    fn sorcery_speed(&self, player_id: PlayerId) -> bool {
        player_id == self.active_player
            && self.stack.is_empty()
            && (self.phase == Some(Phase::FirstMain) || self.phase == Some(Phase::SecondMain))
    }

    pub fn opponents(&self, player: PlayerId) -> Vec<PlayerId> {
        self.turn_order
            .iter()
            .filter_map(|&x| if x == player { None } else { Some(x) })
            .collect()
    }

    pub fn remaining_lethal(&self, ent: CardId) -> Option<i64> {
        self.cards.get(ent).and_then(|card| {
            card.pt
                .as_ref()
                .map(|pt| max(pt.toughness - card.damaged, 0))
        })
    }
    pub fn add_ability(&mut self, ent: CardId, ability: Ability) {
        //Assume the builder has already added a vector of abilities
        if let Some(ent) = self.cards.get_mut(ent) {
            ent.abilities.push(ability);
        }
    }
    //If this is an ability, return its originator. If not, return itself
    pub fn stack_ent_source(&self, id: CardId) -> CardId {
        if let Some(card) = self.cards.get(id) {
            card.source_of_ability.unwrap_or(id)
        } else {
            id
        }
    }
    //Returns true if the event is allowed (not prevented)
    //Source is the attaker/equipper/source of damage
    fn has_protection_from(&self, source: CardId, target: TargetId) -> bool {
        if let TargetId::Card(target)=target
        && let Some(card)=self.cards.get(target){ 
            for abil in &card.abilities{
                if let Ability::Static(abil)=abil
                && let StaticAbilityEffect::Protection(prot)=&abil.effect
                //This is reversed, because perotection checks the charichteristics of the damager, not the target
                && self.passes_constraint(prot, target, source.into())
                {
                    return true;
                }
            }
        }
        return false;
    }
}

pub enum ActionPriorityType {
    Pass,
    ManaAbilOrSpecialAction,
    Action,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum Phase {
    Begin,
    FirstMain,
    Combat,
    SecondMain,
    Ending,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
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
