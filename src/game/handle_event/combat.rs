use std::collections::HashMap;

use crate::{
    client_message::{Ask, AskPairAB},
    entities::{CardId, PlayerId, TargetId},
    event::{DamageReason, Event, EventResult, TagEvent},
    game::{Game, Subphase},
    spellabil::KeywordAbility,
};

impl Game {

    pub fn attack_targets(&self, player: PlayerId) -> Vec<TargetId> {
        self.opponents(player)
            .iter()
            .map(|pl| TargetId::Player(*pl))
            .collect::<Vec<_>>()
    }

    pub async fn attackers(&mut self, _results: &mut Vec<EventResult>, events: &mut Vec<TagEvent>) {
        self.cycle_priority().await;
        self.backup();
        //Only allow creatures that have haste or don't have summoning sickness to attack
        let legal_attackers = self
            .players_creatures(self.active_player)
            .filter(|e| self.can_tap(*e))
            .collect::<Vec<CardId>>();
        let attack_targets = self.attack_targets(self.active_player);
        loop {
            let attacks;
            //Choice limits is inclusive on both bounds
            let a: HashMap<CardId, (usize, usize)> =
                legal_attackers.iter().map(|id| (*id, (0, 1))).collect();
            if let Some(player) = self.players.get(self.active_player) {
                let pairing = AskPairAB {
                    a,
                    b: attack_targets.iter().cloned().collect(),
                };

                attacks = player
                    .ask_user_pair(&Ask::Attackers(pairing.clone()), &pairing)
                    .await;
            } else {
                return;
            }
            let attacks: HashMap<CardId, TargetId> = attacks
                .into_iter()
                .filter_map(|attack| {
                    if attack.1.len() == 0 {
                        None
                    } else {
                        Some((attack.0, attack.1[0]))
                    }
                })
                .collect();
            if !self.attackers_legal(&attacks) {
                self.restore();
                continue;
            }
            for (&attacker, _attacking) in attacks.iter() {
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
            if attacks.len() > 0 {
                for (&attacker, &attacked) in &attacks {
                    if let Some(card) = self.cards.get_mut(attacker) {
                        card.attacking = Some(attacked);
                    }
                }
                events.push(TagEvent {
                    event: Event::Attack { attacks },
                    replacements: Vec::new(),
                });
            } else {
                self.subphases = vec![Subphase::EndCombat].into();
            }
            break;
        }
    }

    pub async fn blockers(&mut self, _results: &mut Vec<EventResult>, events: &mut Vec<TagEvent>) {
        for opponent in self.opponents(self.active_player) {
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
            loop {
                //This will be adjusted for creatures that can make multiple blocks
                let a = potential_blockers.iter().map(|x| (*x, (0, 1))).collect();
                let blocks = if let Some(player) = self.players.get(opponent) {
                    let pairing = AskPairAB {
                        a,
                        b: attacking.iter().cloned().collect(),
                    };
                    player
                        .ask_user_pair(&Ask::Blockers(pairing.clone()), &pairing)
                        .await
                } else {
                    return;
                };
                let mut blockers = Vec::new();
                let mut blocked = Vec::new();
                for (blocker, blocking) in blocks {
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
                break;
            }
        }
        for attacker in self
            .all_creatures()
            .filter(|&creature| self.cards.is(creature, |card| card.attacking.is_some()))
        {
            Game::add_event(events, Event::AttackUnblocked { attacker });
        }
        println!("exiting blockers");
    }

    pub async fn damagephase(
        &mut self,
        _results: &mut Vec<EventResult>,
        events: &mut Vec<TagEvent>,
        subphase: Subphase,
    ) {
        //Handle first strike and normal strike
        for attacker in self.damage_phase_permanents(self.active_player, subphase) {
            if let Some(attack) = self.cards.get_mut(attacker) {
                attack.already_dealt_damage = true;
            }
            if let Some(attack)=self.cards.get(attacker)
            && let Some(attacked)=attack.attacking{
                if attack.blocked.len() > 0 {
                    self.spread_damage(events, attacker, &attack.blocked).await;
                } else {
                    if let Some(pt)=attack.pt{
                        Game::add_event(
                            events,
                            Event::Damage {
                                amount: pt.power,
                                target: attacked,
                                source: attacker,
                                reason: DamageReason::Combat,
                            },
                        );
                    }
                }
            };
        }
        for player in self.opponents(self.active_player) {
            for blocker in self.damage_phase_permanents(player, subphase) {
                if let Some(card) = self.cards.get(blocker) && card.blocking.len()>0 {
                    self.spread_damage(events, blocker, &card.blocking).await;
                }
            }
        }
    }

    pub fn damage_phase_permanents<'b>(
        &'b self,
        player: PlayerId,
        subphase: Subphase,
    ) -> Vec<CardId> {
        self.players_creatures(player)
            .filter(move |&ent| {
                if subphase == Subphase::FirstStrikeDamage {
                    self.cards.has_keyword(ent, KeywordAbility::FirstStrike)
                        || self.cards.has_keyword(ent, KeywordAbility::DoubleStrike)
                } else if subphase == Subphase::Damage {
                    self.cards.has_keyword(ent, KeywordAbility::DoubleStrike)
                        || !self.cards.is(ent, |card| card.already_dealt_damage)
                } else {
                    panic!("This function may only be called within the damage phases")
                }
            })
            .collect()
    }
    pub async fn blocked_by(
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
                let amount = std::cmp::min(damage_to_deal, needed_damage);
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
    //Checks if this attacking arragment is legal.
    //Does nothing for now, will need to implement legality
    //checking before I can make any progress on that
    //This will need to loop over ALL creatures, not just the ones
    //in attacks to handle creatues that must attack
    fn attackers_legal(&self, attacks: &HashMap<CardId, TargetId>) -> bool {
        true
    }
    fn blocks_legal(&self, blockers: &Vec<CardId>, blocked: &Vec<Vec<CardId>>) -> bool {
        true
    }
}
