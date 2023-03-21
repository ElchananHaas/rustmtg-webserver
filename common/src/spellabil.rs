use crate::card_entities::PT;
use crate::cardtypes::{Subtype, Type};
use crate::counters::Counter;
use crate::entities::CardId;
use crate::mana::ManaCostSymbol;
use crate::{entities::TargetId, token_attribute::TokenAttribute};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_derive::Serialize;
use strum_macros::EnumString;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum KeywordAbility {
    FirstStrike,  //Implemented
    Haste,        //Implemented
    Vigilance,    //Implemented
    DoubleStrike, //Implemented
    Flying,       //Implemented
    Prowess,
    Lifelink, //Implemented
    Trample,
    Reach,      //Implemented
    Protection, //Partially implemented, add it can't be enchanted.
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum PermConstraint {
    IsTapped,
    CardType(Type),
    And(Vec<PermConstraint>),
    Or(Vec<PermConstraint>),
    IsCardname,
    YouControl,
    HasKeyword(KeywordAbility),
    Subtype(Subtype),
    HasCounter(Counter),
    Multicolored,
    Other,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct Clause {
    pub effect: ClauseEffect,
    pub affected: Affected,
    pub constraints: Vec<PermConstraint>,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum Affected {
    Controller,
    Cardname,
    All,
    Target(Option<TargetId>),
    ManuallySet(Option<TargetId>),
    UpToXTarget(i64, Vec<TargetId>),
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq)]
pub enum ContDuration {
    Perpetual,
    EndOfTurn,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct Continuous {
    pub effect: ContEffect,
    pub affected: Affected,
    pub constraints: Vec<PermConstraint>,
    pub duration: ContDuration,
    pub source: CardId,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum ContEffect {
    ModifyPT(PT),
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum NumberComputer {
    NumPermanents(Vec<PermConstraint>),
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum ClauseEffect {
    Destroy,
    ExileBattlefield,
    AddMana(Vec<ManaCostSymbol>),
    GainLife(i64),
    DrawCard,
    Compound(Vec<Clause>),
    SetTargetController(Box<Clause>),
    CreateToken(Vec<TokenAttribute>),
    UntilEndTurn(ContEffect),
    MultClause(Box<ClauseEffect>, NumberComputer),
    PutCounter(Counter, i64),
}
