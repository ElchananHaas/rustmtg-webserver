use schemars::JsonSchema;
use serde_derive::Serialize;
use strum_macros::EnumString;
use crate::mana::ManaCostSymbol;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, JsonSchema, EnumString)]
pub enum KeywordAbility {
    FirstStrike,
    Haste,
    Vigilance,
    DoubleStrike,
}

#[derive(Clone, Serialize, JsonSchema)]
pub enum Clause {
    AddMana(Vec<ManaCostSymbol>),
    DrawCard,
}
