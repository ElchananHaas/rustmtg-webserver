use std::collections::{HashMap, HashSet};

use crate::{
    client_message::{Ask, AskPair, AskPairItem},
    event::{DamageReason, Event, EventResult},
    game::{Game, Subphase},
};
use common::spellabil::{ContEffect, KeywordAbility};
use common::{
    entities::{CardId, PlayerId, TargetId},
    hashset_obj::HashSetObj,
};
impl Game {
    pub fn attack_targets(&self, player: PlayerId) -> HashSet<TargetId> {
        self.opponents(player)
            .iter()
            .map(|pl| TargetId::Player(*pl))
            .collect::<HashSet<_>>()
    }

    pub async fn attackers(&mut self, _results: &mut Vec<EventResult>, events: &mut Vec<Event>) {
        self.backup();
        let cant_attack = self.cant_attack();
        //Only allow creatures that have haste or don't have summoning sickness to attack
        let legal_attackers = self
            .players_creatures(self.active_player)
            .filter(|e| self.can_tap(*e) && (!cant_attack.contains(e)))
            .collect::<Vec<CardId>>();
        loop {
            let attacks;
            //Choice limits is inclusive on both bounds
            let pairs = legal_attackers
                .iter()
                .map(|&attacker| {
                    (
                        attacker,
                        AskPairItem {
                            items: self
                                .attack_targets(self.active_player)
                                .into_iter()
                                .collect(),
                            min: 0,
                            max: 1,
                        },
                    )
                })
                .collect();
            if let Some(player) = self.players.get(self.active_player) {
                let pairing = AskPair { pairs };
                attacks = player
                    .ask_user_pair(&Ask::Attackers(pairing.clone()), &pairing)
                    .await;
            } else {
                return;
            }
            let attacks: HashMap<CardId, TargetId> = attacks
                .into_iter()
                .filter_map(|(attack, attacking)| {
                    for x in attacking {
                        return Some((attack, x));
                    }
                    None
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
                events.push(Event::Attack { attacks });
            } else {
                self.subphases = vec![Subphase::EndCombat].into();
            }
            break;
        }
        self.cycle_priority().await;
    }

    pub async fn blockers(&mut self, _results: &mut Vec<EventResult>, events: &mut Vec<Event>) {
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
            let cant_block = self.cant_block();
            let legal_blockers = self
                .players_creatures(opponent)
                .filter(|creature| {
                    self.cards.is(*creature, |card| !card.tapped)
                        && (!cant_block.contains(creature))
                })
                .collect::<Vec<_>>();
            loop {
                //This will be adjusted for creatures that can make multiple blocks
                let pairs = legal_blockers
                    .iter()
                    .map(|&blocker| {
                        let this_can_block: Vec<_> = attacking
                            .clone()
                            .into_iter()
                            .filter(|&attacker| self.can_block(attacker, blocker))
                            .collect();
                        (
                            blocker,
                            AskPairItem {
                                items: this_can_block.iter().cloned().collect(),
                                min: 0,
                                max: 1,
                            },
                        )
                    })
                    .collect();
                let blocks = if let Some(player) = self.players.get(opponent) {
                    let pairing = AskPair { pairs };
                    player
                        .ask_user_pair(&Ask::Blockers(pairing.clone()), &pairing)
                        .await
                } else {
                    return;
                };
                if !self.blocks_legal(&blocks) {
                    self.restore();
                    continue;
                }
                for (blocker, blocking) in blocks {
                    if blocking.len() == 0 {
                        continue;
                    }
                    Game::add_event(events, Event::Block { blocker });
                    for blocked in blocking {
                        Game::add_event(
                            events,
                            Event::BlockedBy {
                                blocker,
                                attacker: blocked,
                            },
                        );
                    }
                }
                break;
            }
        }
        self.cycle_priority().await;
        println!("exiting blockers");
    }

    pub async fn damagephase(
        &mut self,
        _results: &mut Vec<EventResult>,
        events: &mut Vec<Event>,
        subphase: Subphase,
    ) {
        //Handle first strike and normal strike
        let attacks = self.damage_phase_permanents(self.active_player, subphase);
        for &attacker in &attacks {
            if let Some(attack) = self.cards.get_mut(attacker) {
                attack.already_dealt_damage = true;
            }
            if let Some(attack)=self.cards.get(attacker)
            && let Some(attacked)=attack.attacking{
                if attack.blocked.len() > 0 {
                    self.spread_damage(events, attacker, &attack.blocked).await;
                } else {
                    if let Some(pt)=&attack.pt{
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
            for blocker in self.players_creatures(player) {
                if let Some(card) = self.cards.get(blocker) && card.blocking.len()>0 {
                    self.spread_damage(events, blocker, &card.blocking).await;
                }
            }
        }
        if attacks.len() > 0 {
            self.cycle_priority().await;
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
        events: &mut Vec<Event>,
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
        events: &mut Vec<Event>,
        dealer: CardId,
        creatures: &Vec<CardId>,
    ) {
        let mut damage_to_deal =
            if let Some(pt) = self.cards.get(dealer).and_then(|card| card.pt.as_ref()) {
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
    fn cant_attack(&self) -> HashSet<CardId> {
        let mut res = HashSet::new();
        let conts = self.cont_abilities();
        for cont in conts {
            match &cont.effect {
                ContEffect::CantAttackOrBlock => {
                    for affected in
                        self.calculate_affected(cont.source, &cont.affected, &cont.constraints)
                    {
                        if let TargetId::Card(c) = affected {
                            res.insert(c);
                        }
                    }
                }
                _ => {}
            }
        }
        res
    }
    //Checks if this attacking arragment is legal.
    fn attackers_legal(&self, attacks: &HashMap<CardId, TargetId>) -> bool {
        let cant_attack = self.cant_attack();
        for attack in attacks {
            if cant_attack.contains(attack.0) {
                return false;
            }
        }
        true
    }
    //checks if the blocker can legally block the attacker
    fn can_block(&self, attacker: CardId, blocker: CardId) -> bool {
        if self.has_keyword(attacker, KeywordAbility::Flying) {
            if !(self.has_keyword(blocker, KeywordAbility::Flying)
                || self.has_keyword(blocker, KeywordAbility::Reach))
            {
                return false;
            }
        }
        if self.has_protection_from(attacker, blocker.into()) {
            return false;
        }
        true
    }
    fn cant_block(&self) -> HashSet<CardId> {
        let mut res = HashSet::new();
        let conts = self.cont_abilities();
        for cont in conts {
            match &cont.effect {
                ContEffect::CantAttackOrBlock => {
                    for affected in
                        self.calculate_affected(cont.source, &cont.affected, &cont.constraints)
                    {
                        if let TargetId::Card(c) = affected {
                            res.insert(c);
                        }
                    }
                }
                _ => {}
            }
        }
        res
    }
    fn blocks_legal(&self, blocks: &HashMap<CardId, HashSetObj<CardId>>) -> bool {
        for (&blocker, attackers) in blocks {
            for &attacker in attackers {
                if !self.can_block(attacker, blocker) {
                    return false;
                }
            }
        }
        let cant_block = self.cant_attack();
        for block in blocks {
            if cant_block.contains(block.0) {
                return false;
            }
        }
        true
    }
}
