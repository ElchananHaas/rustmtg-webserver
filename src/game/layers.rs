use crate::ability_db::tap_for_mana;
use crate::{game::*, spellabil::SpellAbilBuilder};
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
    pub fn layers(&mut self) {
        self.layer_zero();
        self.layer_four();
    }
    //Handles the printed charachteristics of cards
    //and sets their controller to be their owner
    fn layer_zero(&mut self) {
        for (ent, zone) in self.ents_and_zones() {
            if let Some(card) = self.cards.get_mut(ent) {
                todo!(); //Rebuild from database
                if zone == Zone::Battlefield || zone == Zone::Stack {
                    card.controller = Some(card.owner);
                }
            }
        }
    }
    fn layer_four(&mut self) {
        for (ent, zone) in self.ents_and_zones() {
            if zone == Zone::Battlefield {
                let _: Option<_> = try {
                    let card = self.cards.get_mut(ent)?;
                    if card.subtypes.get(Subtype::Plains) {
                        self.add_ability(ent, tap_for_mana(vec![ManaCostSymbol::White]));
                    }
                };
            }
        }
    }
}
