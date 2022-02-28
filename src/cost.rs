use crate::entities::{CardId, EntId, PlayerId};
use crate::game::Game;
use crate::mana::{Color, ManaCostSymbol};
use crate::player::Player;
use anyhow::{bail, Error, Result};
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
