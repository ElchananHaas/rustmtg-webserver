use crate::mana::ManaCostSymbol;
use schemars::JsonSchema;
use serde_derive::Serialize;
use strum_macros::EnumString;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, JsonSchema, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum KeywordAbility {
    FirstStrike,  //Implemented
    Haste,        //Implemented
    Vigilance,    //Implemented
    DoubleStrike, //Implemented
    Flying,
    Prowess,
    Lifelink,
    Trample,
    Reach,
}

#[derive(Clone, Serialize, JsonSchema, Debug)]
pub enum Clause {
    AddMana(Vec<ManaCostSymbol>),
    DrawCard,
}
