use std::cmp::min;

use crate::event::DamageReason;
use crate::game::*;
use crate::player::{AskReason, Player};
use async_recursion::async_recursion;

impl Game {
    #[async_recursion]
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
                    self.blocked_by(&mut events, attacker, blocker);
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
                Event::Turn { extra: _, player } => {
                    self.active_player = player;
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
                    self.cards.get(ent).map(|card| {
                        if !card.tapped {
                            results.push(EventResult::Tap(ent));
                            card.tapped = true;
                        }
                    });
                }
                //Handle already being untapped as prevention effect
                Event::Untap { ent } => {
                    self.cards.get(ent).map(|card| {
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
    async fn phase(&mut self, _events: &mut Vec<TagEvent>, phase: Phase) {
        self.phase = Some(phase);
        self.subphase = None;
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
    }
    async fn zonemove(
        &mut self,
        results: &mut Vec<EventResult>,
        _events: &mut Vec<TagEvent>,
        ent: CardId,
        origin: Zone,
        dest: Zone,
    ) {
        let props = None;
        try {
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
            //Udate knowledge of new card on zonemove
            if removed && !card.token {
                props = Some((card.name, card.owner));
            };
            if removed && card.token {
                results.push(EventResult::MoveZones {
                    oldent: ent,
                    newent: None,
                    dest,
                });
            }
        };
        try {
            let (name, owner_id) = props?;
            let owner = self.players.get_mut(owner_id)?;
            let newent = self.db.spawn_card(&mut self.cards, name, owner_id);
            let newcard = self.cards.get_mut(newent).unwrap(); //I know this is safe b/c I just spawned it
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
        };
    }
    async fn subphase(
        &mut self,
        results: &mut Vec<EventResult>,
        events: &mut Vec<TagEvent>,
        subphase: Subphase,
    ) {
        self.subphase = Some(subphase);
        match subphase {
            Subphase::Untap => {
                for perm in self
                    .players_permanents(self.active_player)
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
                self.draw(self.active_player).await;
                self.cycle_priority().await;
            }
            Subphase::BeginCombat => {
                self.cycle_priority().await;
            }
            Subphase::Attackers => {
                self.backup();
                //Only allow creatures that have haste or don't have summoning sickness to attack
                let legal_attackers = self
                    .players_creatures(self.active_player)
                    .filter(|e| self.can_tap(*e))
                    .collect::<Vec<Entity>>();
                let attack_targets = self.attack_targets(self.active_player);

                loop {
                    println!("Asking player");
                    let attacks;
                    //Choice limits is inclusive on both bounds
                    let choice_limits = vec![(0, 1); legal_attackers.len()];
                    if let Ok(player) = self.ents.get_mut::<Player>(self.active_player) {
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
                    let mut actual_attackers = Vec::new();
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
                        if !self.has_keyword(attacker, KeywordAbility::Vigilance) {
                            self.tap(attacker).await;
                        }
                    }

                    //Handle costs to attack here
                    //THis may led to a redeclaration of attackers
                    //Now declare them attackers and fire attacking events
                    for (&attacker, &attacked) in actual_attackers.iter().zip(attack_targets.iter())
                    {
                        let _ = self.ents.insert_one(attacker, Attacking(attacked));
                    }
                    events.push(TagEvent {
                        event: Event::Attack {
                            attackers: actual_attackers,
                        },
                        replacements: Vec::new(),
                    });
                    break;
                }
                self.cycle_priority().await;
            }
            Subphase::Blockers => {
                for opponent in self.opponents(self.active_player) {
                    self.backup();
                    //Filter only attacking creatures attacking that player
                    //Add in planeswalkers later
                    let attacking = self
                        .all_creatures()
                        .filter(|&creature| {
                            if let Ok(attack) = self.ents.get::<Attacking>(creature) {
                                attack.0 == opponent
                            } else {
                                false
                            }
                        })
                        .collect::<Vec<_>>();
                    let potential_blockers = self
                        .players_creatures(opponent)
                        .filter(|&creature| !self.has::<Tapped>(creature))
                        .collect::<Vec<_>>();
                    //This will be adjusted for creatres that can make multiple blocks
                    let choice_limits = vec![(0, 1); potential_blockers.len()];
                    loop {
                        let blocks = if let Ok(player) = self.ents.get_mut::<Player>(opponent) {
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
                    .filter(|&creature| self.has::<Attacking>(creature))
                {
                    Game::add_event(events, Event::AttackUnblocked { attacker });
                }
            }
            Subphase::FirstStrikeDamage => self.damagephase(results, events, subphase).await,
            Subphase::Damage => self.damagephase(results, events, subphase).await,
            Subphase::EndCombat => {
                self.cycle_priority().await;
                for perm in self.battlefield.clone() {
                    let _ = self.ents.remove_one::<Attacking>(perm);
                    let _ = self.ents.remove_one::<Blocked>(perm);
                    let _ = self.ents.remove_one::<Blocking>(perm);
                }
            }
            Subphase::EndStep => {
                self.cycle_priority().await;
            }
            Subphase::Cleanup => {
                self.cleanup_phase().await;
            }
        }
    }
    async fn cleanup_phase(&mut self) {
        let mut to_discard = HashSet::new();
        if let Ok(player) = self.ents.get_mut::<Player>(self.active_player) {
            if player.hand.len() > player.max_handsize {
                let diff = player.hand.len() - player.max_handsize;
                let diff: i32 = diff.try_into().unwrap();
                to_discard = player
                    .ask_user_selectn(&player.hand, diff, diff, AskReason::DiscardToHandSize)
                    .await;
            }
        }
        for card in to_discard {
            self.discard(self.active_player, card, DiscardCause::GameInternal)
                .await;
        }
        for perm in self.battlefield.clone() {
            let _ = self.ents.remove_one::<Damage>(perm);
            let _ = self.ents.remove_one::<DealtCombatDamage>(perm);
        }
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
            .damage_phase_permanents(self.players_creatures(self.active_player), subphase)
            .collect::<Vec<_>>()
        {
            let is_attacking = if let Ok(attack) = self.ents.get::<Attacking>(attacker) {
                Some(*attack)
            } else {
                None
            };
            if let Some(unblocked_attack) = is_attacking {
                if let Ok(pt) = self.ents.get::<PT>(attacker) {
                    if pt.power <= 0 {
                        continue;
                    }
                    if let Ok(blocks) = self.ents.get::<Blocked>(attacker) {
                        self.spread_damage(events, attacker, &blocks.0).await;
                    } else {
                        Game::add_event(
                            events,
                            Event::Damage {
                                amount: pt.power,
                                target: unblocked_attack.0,
                                source: attacker,
                                reason: DamageReason::Combat,
                            },
                        );
                    }
                }
            }
        }
        for blocker in self
            .damage_phase_permanents(self.all_creatures(), subphase)
            .collect::<Vec<_>>()
        {
            if let Ok(blocked) = self.ents.get::<Blocking>(blocker) {
                self.spread_damage(events, blocker, &blocked.0).await;
            }
        }
    }
    pub fn damage_phase_permanents<'b>(
        &'b self,
        creatures: impl Iterator<Item = Entity> + 'b,
        subphase: Subphase,
    ) -> impl Iterator<Item = Entity> + 'b {
        creatures.filter(move |&ent| {
            if subphase == Subphase::FirstStrikeDamage {
                self.has_keyword(ent, KeywordAbility::FirstStrike)
                    || self.has_keyword(ent, KeywordAbility::DoubleStrike)
            } else if subphase == Subphase::Damage {
                self.has_keyword(ent, KeywordAbility::DoubleStrike)
                    || !self.has::<DealtCombatDamage>(ent)
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
        dealer: Entity,
        creatures: &Vec<Entity>,
    ) {
        let mut damage_to_deal = if let Ok(pt) = self.ents.get::<PT>(dealer) {
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
                        target: creature,
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
