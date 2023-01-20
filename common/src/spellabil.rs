use crate::mana::{Color, ManaCostSymbol};
use crate::{entities::TargetId, token_attribute::TokenAttribute};
use cardtypes::Type;
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
    Flying,       //Implemented
    Prowess,
    Lifelink,
    Trample,
    Reach, //Implemented
}

#[derive(Clone, Serialize, JsonSchema, Debug)]
pub enum ClauseConstraint {
    IsTapped,
    CardType(Type),
    Or(Vec<ClauseConstraint>),
}

#[derive(Clone, Serialize, JsonSchema, Debug)]
pub struct Clause {
    pub effect: ClauseEffect,
    pub affected: Affected,
    pub constraints: Vec<ClauseConstraint>,
}
#[derive(Clone, Serialize, JsonSchema, Debug, Copy)]
pub enum Affected {
    Controller,
    Target(Option<TargetId>),
    ManuallySet(Option<TargetId>),
}

#[derive(Clone, Serialize, JsonSchema, Debug)]

pub enum ClauseEffect {
    Destroy,
    ExileBattlefield,
    AddMana(Vec<ManaCostSymbol>),
    GainLife(i64),
    DrawCard,
    Compound(Vec<Clause>),
    SetTargetController(Box<Clause>),
    CreateToken(Vec<TokenAttribute>),
}
