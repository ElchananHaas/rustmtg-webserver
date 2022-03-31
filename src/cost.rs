use crate::mana::ManaCostSymbol;
use serde_derive::Serialize;

/*
!!!!!!!!!TODO
Allow the game to interactively ask for costs to be paid
*/
#[derive(Clone, Debug, Serialize)]
pub enum Cost {
    Mana(ManaCostSymbol),
    Selftap,
}
