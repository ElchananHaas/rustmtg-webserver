mod combat;
mod phase_event;

use crate::card_entities::EntType;
use crate::game::*;
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
                Event::Destory { card } => {
                    Game::add_event(
                        &mut events,
                        Event::MoveZones {
                            ent: card,
                            origin: Zone::Battlefield,
                            dest: Zone::Graveyard,
                        },
                    );
                }
                //The assigning as a blocker happens during the two-part block trigger
                Event::Damage {
                    amount,
                    target,
                    source,
                    reason: _,
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
                Event::Turn { player, extra: _ } => {
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
                Event::Attack { attacks: _ } => {}
                Event::Lose { player } => {
                    //TODO add in the logic to have the game terminate such as setting winners
                    todo!();
                }
                Event::MoveZones { ent, origin, dest } => {
                    self.movezones(&mut results, &mut events, ent, origin, dest)
                        .await;
                }
            }
        }
    }
    async fn drain_mana_pools(&mut self) {
        //TODO handle effects that keep mana pools from draining
        for &player in &self.turn_order {
            if let Some(pl) = self.players.get_mut(player) {
                pl.mana_pool = HashSetObj::new();
            }
        }
    }

    async fn movezones(
        &mut self,
        results: &mut Vec<EventResult>,
        _events: &mut Vec<TagEvent>,
        ent: CardId,
        origin: Zone,
        dest: Zone,
    ) {
        let mut props = None;
        if let Some(card)=self.cards.get_mut(ent)
        && let Some(owner)= self.players.get_mut(card.owner) {
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
            let real = card.ent_type == EntType::RealCard;
            if removed && real {
                props = Some((card.name, card.owner));
            };
            if removed && !real {
                results.push(EventResult::MoveZones {
                    oldent: ent,
                    newent: None,
                    source:origin,
                    dest,
                });
            }
        };
        if let Some((name, owner_id))=props 
        && let Some(owner)= self.players.get_mut(owner_id){
            let card = self.db.spawn_card(name, owner_id);
            let (newent, newcard) = self.cards.insert(card);
            //update knowledge of new card on zonemove
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
                    newcard.etb_this_cycle=true;
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
                source: origin,
                dest,
            });
        };
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
    fn add_event(events: &mut Vec<TagEvent>, event: Event) {
        events.push(TagEvent {
            event,
            replacements: Vec::new(),
        });
    }
}
