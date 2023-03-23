use common::zones::Zone;

use crate::game::*;

fn mana_from_clause(clause: &Clause) -> Vec<ManaCostSymbol> {
    if let ClauseEffect::AddMana(mana) = &clause.effect {
        return mana.clone();
    }
    if let ClauseEffect::Compound(clauses) = &clause.effect {
        let mut res = vec![];
        for clause in clauses {
            res.append(&mut mana_from_clause(clause));
        }
        return res;
    }
    vec![]
}

impl Game {
    //For now, only count mana from tap abilities.
    //Add in sac and other abilities later.

    fn mana_tap_abils(&self, ent: CardId) -> Vec<Vec<ManaCostSymbol>> {
        let mut res = vec![];
        if let Some(ent) = self.cards.get(ent) {
            for abil in &ent.abilities {
                if let Ability::Activated(abil) = abil 
                && abil.costs.contains(&Cost::Selftap)
                && !ent.tapped{
                    let mut mana_produced=vec![];
                    for clause in &abil.effect{
                        let mut mana=mana_from_clause(clause);
                        mana_produced.append(&mut mana);
                    }
                    if mana_produced.len()>0 {
                        res.push(mana_produced);
                    }
                }
            }
        }
        res
    }
    fn max_mana_produce(&self, ent: CardId) -> i64 {
        //TODO get more fine grained color support
        let mut mana_produce = 0;
        for manas in self.mana_tap_abils(ent) {
            mana_produce = max(mana_produce, manas.len() as i64);
        }
        mana_produce
    }
    //Don't prompt players to cast spells they can't pay for
    fn maybe_can_pay(&self, costs: &Vec<Cost>, player_id: PlayerId, card_id: CardId) -> bool {
        if let Some(player) = self.players.get(player_id) {
            let mut available_mana: i64 = 0;
            //TODO make this take into account costs more accurately,
            //including handling colors of available mana, no just the quanitity
            for perm in self.players_permanents(player_id) {
                available_mana += self.max_mana_produce(perm);
            }
            available_mana += player.mana_pool.len() as i64;
            for cost in costs {
                let can_pay = match cost {
                    Cost::Selftap => self.battlefield.contains(&card_id) && self.can_tap(card_id),
                    Cost::Mana(_mana) => {
                        if available_mana <= 0 {
                            false
                        } else {
                            available_mana -= 1;
                            true
                        }
                    }
                };
                if !can_pay {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

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
                if card.costs.len() > 0 && (card.types.is_instant() || self.sorcery_speed(player)) {
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
        if card.types.is_land()
            && play_sorcery
            && self.land_play_limit > self.lands_played_this_turn
        {
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
        let controller = card.get_controller();
        for i in 0..card.abilities.len() {
            if zone == Zone::Battlefield && controller == player_id {
                let abil = &card.abilities[i];
                let abil = match abil {
                    Ability::Activated(abil) => abil,
                    _ => continue,
                };
                //TODO handle correct target for restictions 
                if !abil.restrictions.iter().all(|r|self.passes_constraint(r, card_id, card_id.into())){
                    continue;
                }
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
