use crate::{
    entities::{CardId, ManaId},
    mana::ManaCostSymbol,
};
use schemars::JsonSchema;
use serde_derive::Serialize;

/*
!!!!!!!!!TODO
Allow the game to interactively ask for costs to be paid
*/
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
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
