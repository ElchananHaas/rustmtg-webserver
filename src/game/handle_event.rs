use crate::game::*;
use crate::player::{AskReason, Player};
use async_recursion::async_recursion;

impl Game {
    #[async_recursion]
    pub async fn handle_event(&mut self, event: Event, cause: EventCause) -> Vec<EventResult> {
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
                for perm in self.controlled(self.active_player) {
                    self.untap(perm, EventCause::None).await;
                }
                //No run phase bc/ players don't get prioirity normally
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
                self.backup();
                let attackers = self.players_creatures(self.active_player);
                let attack_targets = self.attack_targets(self.active_player);
                if let Ok(mut player) = self.ents.get_mut::<Player>(self.active_player) {
                    let attackers = player
                        .ask_user_pair(attackers.clone(), attack_targets.clone(), AskReason::Attackers)
                        .await;
                    
                }
                self.run_phase();
            }
            Subphase::Blockers => todo!(),
            Subphase::FirstStrikeDamage => todo!(),
            Subphase::Damage => todo!(),
            Subphase::EndCombat => todo!(),
            Subphase::EndStep => {
                self.run_phase();
            },
            Subphase::Cleanup => todo!(),
        }
    }
}
