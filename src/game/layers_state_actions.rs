use crate::game::*;
pub enum Layer {
    OneA, //Copiable effects (Copy, As ETB,)
    OneB, //Face down spells,permanents
    Two,
    Three,
    Four,
    Five,
    Six,
    SevenA, //CDA PT
    SevenB, //set PT to value
    SevenC, //Modify PT
    SevenD, //switch PT
}

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
            let Some(pt)=card.pt{
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
                if zone == Zone::Battlefield || zone == Zone::Stack {
                    card.controller = Some(card.owner);
                } else {
                    card.controller = None;
                }
            }
        }
    }
    fn layer_four(&mut self) {
        for (ent, zone) in self.cards_and_zones() {
            if zone == Zone::Battlefield {
                if let Some(card) = self.cards.get_mut(ent) {
                    let mut abils = Vec::new();
                    if card.subtypes.plains {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::White]));
                    }
                    if card.subtypes.mountain {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::Red]));
                    }
                    if card.subtypes.island {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::Blue]));
                    }
                    if card.subtypes.swamp {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::Black]));
                    }
                    if card.subtypes.forest {
                        abils.push(Ability::tap_for_mana(vec![ManaCostSymbol::Green]));
                    }
                    for abil in abils {
                        self.add_ability(ent, abil);
                    }
                };
            }
        }
    }
}
