use crate::ability_db::tap_for_mana;
use crate::mana::mana_cost_string;
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
macro_rules! make_clear_components {
    ( $( $x:ty ),* ) => {
        fn clear_components(ents: &mut World){
            let entids=ents.iter().map(|entref| entref.entity() ).collect::<Vec<Entity>>();
            for id in entids{
                if ents.get::<EntCore>(id).is_ok(){
                    $(
                        let _=ents.remove_one::<$x>(id);
                    )*
                }
            }
        }

    };
}
make_clear_components! {CardName,PT,Types,Supertypes,Vec<Ability>,HashSet<Subtype>,Controller}

impl Game {
    pub fn layers(&mut self) {
        clear_components(&mut self.ents);
        self.layer_zero();
        self.layer_four();
    }
    //Handles the printed charachteristics of cards
    //and sets their controller to be their owner
    fn layer_zero(&mut self) {
        for (ent, _zone) in self.ents_and_zones() {
            let mut builder = None;
            //Rebuild from database
            if let Ok(core) = self.ents.get::<EntCore>(ent) {
                builder = Some((self.db.layers_builder(&core.name), core.owner));
            }
            //Set it's controller as its owner
            if let Some((mut builder, owner)) = builder {
                let _ = self.ents.insert(ent, builder.build());
                let _ = self.ents.insert_one(ent, owner);
            }
        }
    }
    fn layer_four(&mut self) {
        for (ent, zone) in self.ents_and_zones() {
            if zone == Zone::Battlefield {
                let _: Result<()> = try {
                    let subtypes = self.ents.get::<HashSet<Subtype>>(ent)?;
                    if subtypes.contains(&Subtype::Plains) {
                        self.add_ability(ent, tap_for_mana(vec![ManaCostSymbol::White]));
                    }
                };
            }
        }
    }
}
