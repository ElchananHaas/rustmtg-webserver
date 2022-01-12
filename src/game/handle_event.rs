use crate::components::Attacking;
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
                            self.cycle_priority();
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
                            self.cycle_priority();
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
        _results: &mut Vec<EventResult>,
        events: &mut Vec<TagEvent>,
        subphase: Subphase,
    ) {
        self.subphase = Some(subphase);
        match subphase {
            Subphase::Untap => {
                for perm in self.controlled(self.active_player) {
                    self.untap(perm).await;
                }
                //No run phase bc/ players don't get prioirity normally
            }
            Subphase::Upkeep => {
                self.cycle_priority();
            }
            Subphase::Draw => {
                self.draw(self.active_player).await;
                self.cycle_priority();
            }
            Subphase::BeginCombat => {
                self.cycle_priority();
            }
            Subphase::Attackers => {
                self.backup();
                let legal_attackers = self.players_creatures(self.active_player);
                //Only allow creatures that have haste or don't have summoning sickness to attack
                let legal_attackers = legal_attackers
                    .into_iter()
                    .filter(|e| self.can_tap(*e))
                    .collect::<Vec<Entity>>();
                let attack_targets = self.attack_targets(self.active_player);

                loop {
                    let attacks;
                    //Choice limits is inclusive on lower, exclusive on upper
                    let choice_limits = vec![(0, 2); legal_attackers.len()];
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
                    if !self.attackers_legal(&attacks) {
                        self.restore();
                        continue;
                    }
                    for (i, &attacker) in legal_attackers.iter().enumerate() {
                        let attacked = &attacks[i];
                        if attacked.len() > 0 {
                            let totap =
                                if let Ok(abilities) = self.ents.get::<Vec<Ability>>(attacker) {
                                    !abilities.iter().any(|abil| {
                                        abil.keyword() == Some(KeywordAbility::Vigilance)
                                    })
                                } else {
                                    true
                                };
                            if totap {
                                self.tap(attacker).await;
                            }
                        }
                    }
                    //Handle costs to attack here
                    //THis may led to a redeclaration of attackers
                    //Now declare them attackers and fire attacking events
                    let mut declared = Vec::new();
                    for (i, &attacker) in legal_attackers.iter().enumerate() {
                        let attacked = &attacks[i];
                        if attacked.len() > 0 {
                            let attacked = attacked[0];
                            let _ = self.ents.insert_one(attacker, Attacking(attacked));
                            declared.push(attacker);
                        }
                    }
                    events.push(TagEvent {
                        event: Event::Attack {
                            attackers: declared,
                        },
                        replacements: Vec::new(),
                    });
                    break;
                }

                self.cycle_priority();
            }
            Subphase::Blockers => todo!(),
            Subphase::FirstStrikeDamage => todo!(),
            Subphase::Damage => todo!(),
            Subphase::EndCombat => {
                self.cycle_priority();
            }
            Subphase::EndStep => {
                self.cycle_priority();
            }
            Subphase::Cleanup => {
                let mut to_discard = HashSet::new();
                if let Ok(player) = self.ents.get_mut::<Player>(self.active_player) {
                    if player.hand.len() > player.max_handsize {
                        let diff = player.hand.len() - player.max_handsize;
                        let diff: i32 = diff.try_into().unwrap();
                        to_discard = player
                            .ask_user_selectn(
                                &player.hand,
                                diff,
                                diff + 1,
                                AskReason::DiscardToHandSize,
                            )
                            .await;
                    }
                }
                for card in to_discard {
                    self.discard(self.active_player, card, DiscardCause::GameInternal).await;
                }
                //TODO clean up damage
                //TODO handle priority being given in cleanup step by giving
                //another cleanup step afterwards
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
