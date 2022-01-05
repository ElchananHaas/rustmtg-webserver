//This file stores varius components that entities may 
use serde_derive::Serialize;
use hecs::{Entity};
use std::collections::HashSet;
//Utility structure for figuring out if a creature can tap
//Added the turn it ETBs or changes control
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct SummoningSickness();
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Tapped();
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct CardName(pub String);
#[derive(Clone, Debug, Serialize)]
pub struct EntCore {
    pub owner: Entity,
    pub name: String,
    pub real_card: bool,
    pub known: HashSet<Entity>,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PT {
    pub power: i32,
    pub toughness: i32,
}
