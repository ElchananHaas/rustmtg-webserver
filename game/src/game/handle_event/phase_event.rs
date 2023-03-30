use crate::{
    client_message::{Ask, AskSelectN},
    event::{EventResult, Event},
    game::{Game, Phase, Subphase},
};
use common::{entities::CardId, spellabil::ContDuration};

impl Game {
    pub async fn phase(&mut self, _events: &mut Vec<Event>, phase: Phase) {
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

    pub async fn subphase(
        &mut self,
        results: &mut Vec<EventResult>,
        events: &mut Vec<Event>,
        subphase: Subphase,
    ) {
        self.subphase = Some(subphase);
        self.send_state().await;
        match subphase {
            Subphase::Untap => {
                for perm in self
                    .players_permanents(self.active_player)
                    .collect::<Vec<_>>()
                {
                    self.untap(perm).await;
                    if let Some(card) = self.cards.get_mut(perm) {
                        card.etb_this_cycle = false;
                    }
                }
                //Don't cycle priority in untap
            }
            Subphase::Upkeep => self.cycle_priority().await,
            Subphase::Draw => {
                self.draw(self.active_player).await;
                self.cycle_priority().await
            }
            Subphase::BeginCombat => self.cycle_priority().await,
            Subphase::Attackers => self.attackers(results, events).await,
            Subphase::Blockers => self.blockers(results, events).await,
            Subphase::FirstStrikeDamage => self.damagephase(results, events, subphase).await,
            Subphase::Damage => self.damagephase(results, events, subphase).await,
            Subphase::EndCombat => {
                self.cycle_priority().await;
                for &perm in &self.battlefield {
                    if let Some(card) = self.cards.get_mut(perm) {
                        card.attacking = None;
                        card.blocked = Vec::new();
                        card.blocking = Vec::new();
                        card.already_dealt_damage = false;
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
        if let Some(player) = self.players.get_mut(self.active_player) {
            if player.hand.len() > player.max_handsize {
                let diff = player.hand.len().saturating_sub(player.max_handsize);
                let diff = diff.try_into().expect("handisize fits into i64");
                let hand: Vec<CardId> = player.hand.iter().cloned().collect();
                let ask = AskSelectN {
                    ents: hand.clone(),
                    min: diff,
                    max: diff,
                };
                let to_discard = player
                    .ask_user_selectn(&Ask::DiscardToHandSize(ask.clone()), &ask)
                    .await;
                let to_discard = to_discard.into_iter().map(|i| hand[i]).collect();
                self.discard(self.active_player, to_discard).await;
            }
        }

        for &perm in &self.battlefield {
            if let Some(perm) = self.cards.get_mut(perm) {
                perm.damaged = 0;
            }
        }
        self.lands_played_this_turn = 0;
        //Remove until end of turn effects
        self.cont_effects = self
            .cont_effects
            .clone()
            .into_iter()
            .filter(|effect| effect.duration != ContDuration::EndOfTurn)
            .collect();
        //TODO handle priority being given in cleanup step by giving
        //another cleanup step afterwards
    }
}
