use std::cmp::min;

use crate::card_entities::EntType;
use crate::event::DamageReason;
use crate::game::*;
use crate::player::AskReason;
use async_recursion::async_recursion;

impl Game {
    /*
    This function should
    be  refactored to have stages, starting with prevention effects and
    going on to replacments, then finally
    the event can be handled
    */
    #[async_recursion]
    #[must_use]
    pub async fn handle_event(&mut self, event: Event) -> Vec<EventResult> {
        let mut results: Vec<EventResult> = Vec::new();
        let mut events: Vec<TagEvent> = Vec::new();
        events.push(TagEvent {
            event,
            replacements: Vec::new(),
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
                //The assigning as a blocker happens during the two-part block trigger
                Event::Damage {
                    amount,
                    target,
                    source,
                    reason,
                } => {
                    self.handle_damage(amount, target, source).await;
                }
                Event::Block { blocker: _ } => {}
                Event::BlockedBy { attacker, blocker } => {
                    self.blocked_by(&mut events, attacker, blocker).await;
                }
                Event::Blocked { attacker: _ } => {}
                Event::AttackUnblocked { attacker: _ } => {}
                Event::Discard {
                    player: _,
                    card,
                    cause: _,
                } => {
                    Game::add_event(
                        &mut events,
                        Event::MoveZones {
                            ent: card,
                            origin: Zone::Hand,
                            dest: Zone::Graveyard,
                        },
                    );
                }
                Event::PlayLand { player, land } => {
                    if let Some(zone) = self.locate_zone(land) {
                        self.lands_played_this_turn += 1;
                        Game::add_event(
                            &mut events,
                            Event::MoveZones {
                                ent: land,
                                origin: zone,
                                dest: Zone::Battlefield,
                            },
                        )
                    }
                }
                Event::Turn { extra: _ } => {
                    self.turn_order.rotate_left(1);
                    println!("starting turn");
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
                    self.subphase(&mut results, &mut events, subphase).await;
                }
                Event::Phase { phase } => {
                    self.phase(&mut events, phase).await;
                }
                //Handle already being tapped as prevention effect
                Event::Tap { ent } => {
                    if !self.battlefield.contains(&ent) {
                        continue;
                    }
                    self.cards.get_mut(ent).map(|card| {
                        if !card.tapped {
                            results.push(EventResult::Tap(ent));
                            card.tapped = true;
                        }
                    });
                }
                //Handle already being untapped as prevention effect
                Event::Untap { ent } => {
                    if !self.battlefield.contains(&ent) {
                        continue;
                    }
                    self.cards.get_mut(ent).map(|card| {
                        if card.tapped {
                            results.push(EventResult::Untap(ent));
                            card.tapped = false;
                        }
                    });
                }
                Event::Draw { player } => {
                    if let Some(pl) = self.players.get_mut(player) {
                        match pl.library.last() {
                            Some(card) => {
                                Game::add_event(
                                    &mut events,
                                    Event::MoveZones {
                                        ent: *card,
                                        origin: Zone::Library,
                                        dest: Zone::Hand,
                                    },
                                );
                                results.push(EventResult::Draw(*card));
                            }
                            None => Game::add_event(&mut events, Event::Lose { player: player }),
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
                //They have already been declared attackers by now,
                //and being declared an attacker can't be replaced
                //so this event is just for triggers
                Event::Attack { attackers: _ } => {}
                Event::Lose { player } => {
                    //TODO add in the logic to have the game terminate such as setting winners
                    todo!();
                }
                Event::MoveZones { ent, origin, dest } => {
                    self.zonemove(&mut results, &mut events, ent, origin, dest)
                        .await;
                }
            }
        }
    }
    async fn drain_mana_pools(&mut self) {
        //TODO handle effects that keep mana pools from draining
        for &player in &self.turn_order {
            if let Some(pl) = self.players.get_mut(player) {
                pl.mana_pool = HashSet::new();
            }
        }
    }
    async fn phase(&mut self, _events: &mut Vec<TagEvent>, phase: Phase) {
        self.phase = Some(phase);
        self.subphase = None;
        self.send_state().await;
        match phase {
            Phase::Begin => {
                self.subphases
                    .extend([Subphase::Untap, Subphase::Upkeep, Subphase::Draw].iter());
            }
            Phase::FirstMain => {
                self.cycle_priority().await;
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
                self.cycle_priority().await;
            }
            Phase::Ending => {
                self.subphases
                    .extend([Subphase::EndStep, Subphase::Cleanup].iter());
            }
        }
        self.drain_mana_pools().await;
    }
    async fn zonemove(
        &mut self,
        results: &mut Vec<EventResult>,
        _events: &mut Vec<TagEvent>,
        ent: CardId,
        origin: Zone,
        dest: Zone,
    ) {
        let mut props = None;
        let _: Option<_> = try {
            let card = self.cards.get_mut(ent)?;
            let owner = self.players.get_mut(card.owner)?;
            let removed = match origin {
                Zone::Exile => self.exile.remove(&ent),
                Zone::Command => self.command.remove(&ent),
                Zone::Battlefield => self.battlefield.remove(&ent),
                Zone::Hand => owner.hand.remove(&ent),
                Zone::Library => match owner.library.iter().position(|x| *x == ent) {
                    Some(i) => {
                        owner.library.remove(i);
                        true
                    }
                    None => false,
                },
                Zone::Graveyard => match owner.graveyard.iter().position(|x| *x == ent) {
                    Some(i) => {
                        owner.graveyard.remove(i);
                        true
                    }
                    None => false,
                },
                Zone::Stack => match self.stack.iter().position(|x| *x == ent) {
                    Some(i) => {
                        self.stack.remove(i);
                        true
                    }
                    None => false,
                },
            };
            //date knowledge of new card on zonemove
            let real = card.ent_type == EntType::RealCard;
            if removed && real {
                props = Some((card.name, card.owner));
            };
            if removed && !real {
                results.push(EventResult::MoveZones {
                    oldent: ent,
                    newent: None,
                    dest,
                });
            }
            Some(())
        };
        let _: Option<_> = try {
            let (name, owner_id) = props?;
            let owner = self.players.get_mut(owner_id)?;
            let card = self.db.spawn_card(name, owner_id);
            let (newent, newcard) = self.cards.insert(card);
            match dest {
                Zone::Exile | Zone::Stack | Zone::Command | Zone::Battlefield | Zone::Graveyard => {
                    newcard.known_to.extend(self.turn_order.iter());
                    //Public zone
                }
                Zone::Hand => {
                    newcard.known_to.insert(newcard.owner);
                } //Shuffling will destroy all knowledge of cards in the library
                _ => {}
            }
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
                    owner.hand.insert(newent);
                }
                //Handle inserting a distance from the top. Perhaps swap them afterwards?
                Zone::Library => owner.library.push(newent),
                Zone::Graveyard => owner.graveyard.push(newent),
                Zone::Stack => self.stack.push(newent),
            }
            results.push(EventResult::MoveZones {
                oldent: ent,
                newent: Some(newent),
                dest,
            });
            Some(())
        };
    }
    async fn subphase(
        &mut self,
        results: &mut Vec<EventResult>,
        events: &mut Vec<TagEvent>,
        subphase: Subphase,
    ) {
        self.subphase = Some(subphase);
        self.send_state().await;
        match subphase {
            Subphase::Untap => {
                for perm in self
                    .players_permanents(self.active_player())
                    .collect::<Vec<_>>()
                {
                    self.untap(perm).await;
                }
                //Don't cycle priority in untap
            }
            Subphase::Upkeep => {
                self.cycle_priority().await;
            }
            Subphase::Draw => {
                self.draw(self.active_player()).await;
                self.cycle_priority().await;
            }
            Subphase::BeginCombat => {
                self.cycle_priority().await;
            }
            Subphase::Attackers => {
                self.backup();
                //Only allow creatures that have haste or don't have summoning sickness to attack
                let legal_attackers = self
                    .players_creatures(self.active_player())
                    .filter(|e| self.can_tap(*e))
                    .collect::<Vec<CardId>>();
                let attack_targets = self.attack_targets(self.active_player());
                let mut actual_attackers = Vec::new();
                loop {
                    println!("Asking player");
                    let attacks;
                    //Choice limits is inclusive on both bounds
                    let choice_limits = vec![(0, 1); legal_attackers.len()];
                    if let Some(player) = self.players.get(self.active_player()) {
                        attacks = player
                            .ask_user_pair(
                                legal_attackers.clone(),
                                attack_targets.clone(),
                                choice_limits,
                                AskReason::Attackers,
                            )
                            .await;
                    } else {
                        return;
                    }
                    let mut attack_targets = Vec::new();
                    for (i, &attacker) in legal_attackers.iter().enumerate() {
                        let attacked = &attacks[i];
                        if attacked.len() > 0 {
                            actual_attackers.push(attacker);
                            attack_targets.push(attacked[0]);
                        }
                    }
                    if !self.attackers_legal(&actual_attackers, &attack_targets) {
                        self.restore();
                        continue;
                    }
                    for &attacker in actual_attackers.iter() {
                        if !self
                            .cards
                            .is(attacker, |card| card.has_keyword(KeywordAbility::Vigilance))
                        {
                            self.tap(attacker).await;
                        }
                    }

                    //Handle costs to attack here
                    //THis may led to a redeclaration of attackers
                    //Now declare them attackers and fire attacking events
                    for (&attacker, &attacked) in actual_attackers.iter().zip(attack_targets.iter())
                    {
                        self.cards
                            .get_mut(attacker)
                            .map(|card| card.attacking = Some(attacked));
                    }
                    events.push(TagEvent {
                        event: Event::Attack {
                            attackers: actual_attackers.clone(),
                        },
                        replacements: Vec::new(),
                    });
                    break;
                }
                if actual_attackers.len() == 0 {
                    self.subphases = vec![Subphase::EndCombat].into();
                } else {
                    self.cycle_priority().await;
                }
            }
            Subphase::Blockers => {
                for opponent in self.opponents(self.active_player()) {
                    self.backup();
                    //Filter only attacking creatures attacking that player
                    //Add in planeswalkers later
                    let attacking = self
                        .all_creatures()
                        .filter(|&creature| {
                            self.cards
                                .is(creature, |card| card.attacking == Some(opponent.into()))
                        })
                        .collect::<Vec<_>>();
                    let potential_blockers = self
                        .players_creatures(opponent)
                        .filter(|&creature| self.cards.is(creature, |card| !card.tapped))
                        .collect::<Vec<_>>();
                    //This will be adjusted for creatures that can make multiple blocks
                    let choice_limits = vec![(0, 1); potential_blockers.len()];
                    loop {
                        let blocks = if let Some(player) = self.players.get(opponent) {
                            player
                                .ask_user_pair(
                                    potential_blockers.clone(),
                                    attacking.clone(),
                                    choice_limits.clone(),
                                    AskReason::Blockers,
                                )
                                .await
                        } else {
                            return;
                        };
                        let mut blockers = Vec::new();
                        let mut blocked = Vec::new();
                        for (i, &blocker) in potential_blockers.iter().enumerate() {
                            let blocking = &blocks[i];
                            if blocking.len() > 0 {
                                blockers.push(blocker);
                                blocked.push(blocking.clone());
                            }
                        }
                        if !self.blocks_legal(&blockers, &blocked) {
                            self.restore();
                            continue;
                        }
                        for (i, &blocker) in blockers.iter().enumerate() {
                            Game::add_event(events, Event::Block { blocker });
                            for itsblocks in blocked[i].iter() {
                                Game::add_event(
                                    events,
                                    Event::BlockedBy {
                                        blocker,
                                        attacker: *itsblocks,
                                    },
                                );
                            }
                        }
                    }
                }
                for attacker in self
                    .all_creatures()
                    .filter(|&creature| self.cards.is(creature, |card| card.attacking.is_some()))
                {
                    Game::add_event(events, Event::AttackUnblocked { attacker });
                }
            }
            Subphase::FirstStrikeDamage => self.damagephase(results, events, subphase).await,
            Subphase::Damage => self.damagephase(results, events, subphase).await,
            Subphase::EndCombat => {
                self.cycle_priority().await;
                for &perm in &self.battlefield {
                    if let Some(card) = self.cards.get_mut(perm) {
                        card.attacking = None;
                        card.blocked = Vec::new();
                        card.blocking = Vec::new();
                        card.already_attacked = false;
                    }
                }
            }
            Subphase::EndStep => {
                self.cycle_priority().await;
            }
            Subphase::Cleanup => {
                self.cleanup_phase().await;
            }
        }
        self.drain_mana_pools().await;
    }
    async fn cleanup_phase(&mut self) {
        if let Some(player) = self.players.get_mut(self.active_player()) {
            if player.hand.len() > player.max_handsize {
                let diff = player.hand.len().saturating_sub(player.max_handsize);
                let diff: u32 = diff.try_into().unwrap();
                let hand: Vec<CardId> = player.hand.iter().cloned().collect();
                let to_discard = player
                    .ask_user_selectn(&hand, diff, diff, AskReason::DiscardToHandSize)
                    .await;
                for i in to_discard {
                    self.discard(self.active_player(), hand[i], DiscardCause::GameInternal)
                        .await;
                }
            }
        }

        for &perm in &self.battlefield {
            if let Some(perm) = self.cards.get_mut(perm) {
                perm.damaged = 0;
            }
        }
        self.lands_played_this_turn = 0;
        //TODO handle priority being given in cleanup step by giving
        //another cleanup step afterwards
    }
    async fn damagephase(
        &mut self,
        _results: &mut Vec<EventResult>,
        events: &mut Vec<TagEvent>,
        subphase: Subphase,
    ) {
        //Handle first strike and normal strike
        for attacker in self
            .damage_phase_permanents(self.players_creatures(self.active_player()), subphase)
            .collect::<Vec<_>>()
        {
            let _: Option<_> = try {
                {
                    let attack = self.cards.get_mut(attacker)?;
                    attack.already_attacked = true;
                }
                let attack = self.cards.get(attacker)?;
                let target = attack.attacking?;
                if attack.blocked.len() > 0 {
                    self.spread_damage(events, attacker, &attack.blocked).await;
                } else {
                    Game::add_event(
                        events,
                        Event::Damage {
                            amount: attack.pt?.power,
                            target,
                            source: attacker,
                            reason: DamageReason::Combat,
                        },
                    );
                }
                Some(())
            };
        }
        for blocker in self.all_creatures() {
            if let Some(card) = self.cards.get(blocker) {
                self.spread_damage(events, blocker, &card.blocking).await;
            }
        }
    }
    pub fn damage_phase_permanents<'b>(
        &'b self,
        creatures: impl Iterator<Item = CardId> + 'b,
        subphase: Subphase,
    ) -> impl Iterator<Item = CardId> + 'b {
        creatures.filter(move |&ent| {
            if subphase == Subphase::FirstStrikeDamage {
                self.cards.has_keyword(ent, KeywordAbility::FirstStrike)
                    || self.cards.has_keyword(ent, KeywordAbility::DoubleStrike)
            } else if subphase == Subphase::Damage {
                self.cards.has_keyword(ent, KeywordAbility::DoubleStrike)
                    || !self.cards.is(ent, |card| card.already_attacked)
            } else {
                panic!("This function may only be called within the damage phases")
            }
        })
    }
    async fn blocked_by(
        &mut self,
        events: &mut Vec<TagEvent>,
        attacker_id: CardId,
        blocker_id: CardId,
    ) {
        if let Some(attacker) = self.cards.get_mut(attacker_id) {
            if attacker.blocked.len() == 0 {
                Game::add_event(
                    events,
                    Event::Blocked {
                        attacker: attacker_id,
                    },
                );
            }
            attacker.blocked.push(blocker_id);
        }
        if let Some(blocker) = self.cards.get_mut(blocker_id) {
            blocker.blocking.push(attacker_id);
        }
    }
    //Add deathtouch and combat triggers
    async fn handle_damage(&mut self, amount: i64, target: TargetId, source: CardId) {
        if amount <= 0 {
            return;
        }
        match target {
            TargetId::Card(cardid) => {
                if let Some(card) = self.cards.get_mut(cardid) {
                    card.damaged += amount;
                }
            }
            TargetId::Player(playerid) => {
                if let Some(player) = self.players.get_mut(playerid) {
                    player.life -= amount;
                }
            }
        }
    }
    async fn spread_damage(
        &self,
        events: &mut Vec<TagEvent>,
        dealer: CardId,
        creatures: &Vec<CardId>,
    ) {
        let mut damage_to_deal = if let Some(pt) = self.cards.get(dealer).and_then(|card| card.pt) {
            pt.power
        } else {
            return;
        };
        if damage_to_deal <= 0 {
            return;
        }
        for &creature in creatures {
            if damage_to_deal <= 0 {
                break;
            }
            if let Some(needed_damage) = self.remaining_lethal(creature) {
                let amount = min(damage_to_deal, needed_damage);
                Game::add_event(
                    events,
                    Event::Damage {
                        amount,
                        target: creature.into(),
                        source: dealer,
                        reason: DamageReason::Combat,
                    },
                );
                damage_to_deal -= amount;
            }
        }
    }
    fn add_event(events: &mut Vec<TagEvent>, event: Event) {
        events.push(TagEvent {
            event,
            replacements: Vec::new(),
        });
    }
}
