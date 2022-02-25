use crate::entities::{CardId, EntId, PlayerId};
use crate::game::Game;
use crate::mana::{Color, ManaCostSymbol};
use crate::player::Player;
use anyhow::{bail, Error, Result};
use serde_derive::Serialize;

/*
!!!!!!!!!TODO
Fix this to check that the cost obligations are
fulfilled by the supplied mana. This should enable
hybrid mana with ease
*/
#[derive(Clone, Debug, Serialize)]
pub enum Cost {
    Mana(ManaCostSymbol),
    Selftap,
}