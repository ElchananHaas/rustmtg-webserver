
use schemars::JsonSchema;
use serde_derive::Serialize;

use crate::{
    mana::ManaCostSymbol,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, JsonSchema)]
pub enum KeywordAbility {
    FirstStrike,
    Haste,
    Vigilance,
    DoubleStrike,
}


#[derive(Clone, Serialize, JsonSchema)]
pub enum Clause {
    AddMana(Vec<ManaCostSymbol>),
    DrawCard
}

