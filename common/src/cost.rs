use crate::entities::{CardId, ManaId};
use crate::mana::ManaCostSymbol;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_derive::Serialize;

/*
!!!!!!!!!TODO
Allow the game to interactively ask for costs to be paid
*/
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum Cost {
    Mana(ManaCostSymbol),
    Selftap,
}
#[derive(Debug, Clone, Copy)]
pub enum PaidCost {
    Tapped(CardId),
    PaidMana(ManaId),
}
impl Cost {
    pub fn is_mana(&self) -> bool {
        if let Cost::Mana(_) = self {
            true
        } else {
            false
        }
    }
}
