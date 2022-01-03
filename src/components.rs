//This file stores varius components that entities may 
use serde::Serialize;
use serde_derive::Serialize;
use hecs::{Entity};
//Utility structure for figuring out if a creature can tap
//Added the turn it ETBs or changes control
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct SummoningSickness();
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Tapped();
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Owner(pub Entity);
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct CardName(pub String);
#[derive(Clone, Debug, Serialize)]
pub struct CardIdentity {
    pub name: String,
    pub token: bool,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PT {
    pub power: i32,
    pub toughness: i32,
}
