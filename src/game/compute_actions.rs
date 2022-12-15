use crate::game::*;

impl Game {
    pub fn compute_actions(&self, player: PlayerId) -> Vec<Action> {
        let mut actions = Vec::new();
        if let Some(pl) = self.players.get(player) {
            for &card_id in &pl.hand {
                if let Some(card) = self.cards.get(card_id) {
                    actions.extend(self.play_land_actions(player, card_id, card));
                }
            }
            for (card_id, zone) in self.cards_and_zones() {
                if let Some(card) = self.cards.get(card_id) {
                    actions.extend(self.ability_actions(player, card_id, card, zone));
                }
            }
            actions.extend(self.cast_actions(pl, player));
        }
        actions
    }

    pub fn cast_actions(&self, pl: &Player, player: PlayerId) -> Vec<Action> {
        let mut actions = Vec::new();
        for &card_id in pl.hand.iter() {
            if let Some(card) = self.cards.get(card_id) {
                if card.costs.len() > 0 && (card.types.instant || self.sorcery_speed(player)) {
                    let maybe_pay = self.maybe_can_pay(&card.costs, player, card_id);
                    actions.push(Action::Cast(CastingOption {
                        source_card: card_id,
                        costs: card.costs.clone(),
                        filter: ActionFilter::None,
                        zone: Zone::Hand,
                        player,
                        possible_to_take: maybe_pay,
                    }));
                }
            }
        }
        actions
    }

    fn play_land_actions(
        &self,
        player_id: PlayerId,
        card_id: CardId,
        card: &CardEnt,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let play_sorcery = self.sorcery_speed(player_id);
        if card.types.land && play_sorcery && self.land_play_limit > self.lands_played_this_turn {
            actions.push(Action::PlayLand(card_id));
        }
        actions
    }

    fn ability_actions(
        &self,
        player_id: PlayerId,
        card_id: CardId,
        card: &CardEnt,
        zone: Zone,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let controller = card.controller.unwrap_or(card.owner);
        for i in 0..card.abilities.len() {
            if zone == Zone::Battlefield && controller == player_id {
                let abil = &card.abilities[i];
                let abil = match abil {
                    Ability::Activated(abil) => abil,
                    _ => continue,
                };

                let maybe_pay = self.maybe_can_pay(&abil.costs, player_id, card_id);
                if !maybe_pay {
                    continue;
                }
                actions.push(Action::ActivateAbility {
                    source: card_id,
                    index: i,
                })
            }
        }
        actions
    }
}
