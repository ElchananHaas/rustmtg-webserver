use std::cmp::min;

use crate::components::{Attacking, Blocked, Blocking, DealtCombatDamage};
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
            event: event,
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
                    if amount <= 0 {
                        continue;
                    }
                    if reason == DamageReason::Combat {
                        let _ = self.ents.insert_one(source, DealtCombatDamage());
                    }
                    let already_damaged =
                        if let Ok(mut damage) = self.ents.get_mut::<Damage>(target) {
                            damage.0 += amount;
                            true
                        } else {
                            false
                        };
                    if !already_damaged {
                        let _ = self.ents.insert_one(target, Damage(amount));
                    };
                }
                Event::Block { blocker: _ } => {}
                Event::BlockedBy { attacker, blocker } => {
                    let unblocked = self.ents.get::<Blocked>(attacker).is_err();
                    if unblocked {
                        let _ = self.ents.insert_one(attacker, Blocked(Vec::new()));
                        Game::add_event(&mut events, Event::Blocked { attacker });
                    }
                    if let Ok(mut blockedby) = self.ents.get_mut::<Blocked>(attacker) {
                        blockedby.0.push(blocker);
                    }
                    let noblock = self.ents.get::<Blocking>(blocker).is_err();
                    if noblock {
                        let _ = self.ents.insert_one(blocker, Blocking(Vec::new()));
                    }
                    if let Ok(mut blocking) = self.ents.get_mut::<Blocking>(blocker) {
                        blocking.0.push(attacker);
                    }
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
                Event::Draw { player } => {
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
                    if let Ok(mut pl) = self.ents.get_mut::<Player>(player) {
                        (*pl).lost = true;
                    }
                }
                Event::MoveZones { ent, origin, dest } => {
                    self.zonemove(&mut results, &mut events, ent, origin, dest)
                        .await;
                }
            }
        }
    }
    async fn zonemove(
        &mut self,
        results: &mut Vec<EventResult>,
        _events: &mut Vec<TagEvent>,
        ent: Entity,
        origin: Zone,
        dest: Zone,
    ) {
        if origin == dest {
            return;
        };
        let core = &mut None;
        {
            if let Ok(coreborrow) = self.ents.get::<EntCore>(ent) {
                let refentcore = &(*coreborrow);
                *core = Some(refentcore.clone());
            }
        }
        let mut removed = false;
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
                    Zone::Graveyard => match player.graveyard.iter().position(|x| *x == ent) {
                        Some(i) => {
                            player.graveyard.remove(i);
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
        //TODO clean up damage
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
