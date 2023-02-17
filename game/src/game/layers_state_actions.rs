use common::{counters::Counter, spellabil::ContEffect};

use crate::game::*;

impl Game {
    //Computes layers, state based actions and places abilities on the stack
    pub async fn layers_state_actions(&mut self) {
        self.layers();
        self.state_based_actions().await;
        self.place_abilities().await;
    }
    async fn state_based_actions(&mut self) {
        for &cardid in &self.battlefield.clone() {
            if let Some(card)=self.cards.get(cardid) &&
            let Some(pt)=&card.pt{
                if pt.toughness<=0{
                    self.move_zones(cardid, Zone::Battlefield, Zone::Graveyard).await;
                }
                else if card.damaged>=pt.toughness{
                    self.destroy(cardid).await;
                }
            }
        }
    }
    //Places abilities on the stack
    async fn place_abilities(&mut self) {
        //TODO!
    }
    fn layers(&mut self) {
        self.layer_zero();
        self.layer_four();
        self.layer_seven();
    }
    //Handles the printed charachteristics of cards
    //and sets their controller to be their owner
    //This function perserves all aspects not
    //explicitly set here
    fn layer_zero(&mut self) {
        self.land_play_limit = 1;
        for (ent, zone) in self.cards_and_zones() {
            if let Some(card) = self.cards.get_mut(ent) {
                let base = card.printed.as_ref().unwrap().as_ref().clone();
                card.types = base.types;
                card.subtypes = base.subtypes;
                card.supertypes = base.supertypes;
                card.name = base.name;
                card.abilities = base.abilities;
                card.costs = base.costs;
                card.pt = base.pt;
                if zone == Zone::Battlefield || zone == Zone::Stack {
                    card.set_controller(Some(card.owner));
                } else {
                    card.set_controller(None);
                }
            }
        }
    }
    fn layer_four(&mut self) {
        for (ent, zone) in self.cards_and_zones() {
            if zone == Zone::Battlefield {
                if let Some(card) = self.cards.get_mut(ent) {
                    let mut abils = Vec::new();
                    if card.subtypes.contains(&Subtype::Plains) {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::White]));
                    }
                    if card.subtypes.contains(&Subtype::Mountain) {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::Red]));
                    }
                    if card.subtypes.contains(&Subtype::Island) {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::Blue]));
                    }
                    if card.subtypes.contains(&Subtype::Swamp) {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::Black]));
                    }
                    if card.subtypes.contains(&Subtype::Forest) {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::Green]));
                    }
                    for abil in abils {
                        self.add_ability(ent, abil);
                    }
                };
            }
        }
    }
    fn layer_seven(&mut self) {
        for effect in self.cont_effects.clone() {
            match effect.effect {
                ContEffect::ModifyPT(pt) => {
                    for affected in self
                        .calculate_affected(effect.source, &effect.affected, &effect.constraints)
                        .clone()
                    {
                        if let TargetId::Card(id)=affected
                            && let Some(card)=self.cards.get_mut(id)
                            && let Some(card_pt)=&mut card.pt{
                                card_pt.power+=pt.power;
                                card_pt.toughness+=pt.toughness;
                                println!("{:?}",card_pt.power);
                            }
                    }
                }
            }
        }
        for id in self.battlefield.clone() {
            if let Some(card) = self.cards.get_mut(id) {
                if let Some(mut pt) = card.pt.as_mut() {
                    for counter in &card.counters {
                        match counter {
                            Counter::Plus1Plus1 => {
                                pt.power += 1;
                                pt.toughness += 1;
                            }
                        }
                    }
                }
            }
        }
        let preventions = Vec::new();
        self.prevention_effects = preventions;
    }
}
