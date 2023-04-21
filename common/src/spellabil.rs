use crate::ability::Ability;
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
    Flash,
    Enchant,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum Constraint {
    IsTapped,
    CardType(Type),
    And(Vec<Constraint>),
    Or(Vec<Constraint>),
    IsCardname,
    YouControl,
    ControlWith(Vec<Constraint>, i64),
    HasKeyword(KeywordAbility),
    Subtype(Subtype),
    HasCounter(Counter),
    Multicolored,
    NonToken,
    NotCast,
    Permanent,
    Other,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct Clause {
    pub effect: ClauseEffect,
    pub affected: Affected,
    pub constraints: Vec<Constraint>,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum Affected {
    Controller,
    Cardname,
    All,
    Target(Option<TargetId>),
    ManuallySet(Vec<TargetId>),
    UpToXTarget(i64, Vec<TargetId>),
    EquippedOrEnchanted,
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
    pub constraints: Vec<Constraint>,
    pub duration: ContDuration,
    pub source: CardId,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum ContEffect {
    ModifyPT(PT),
    HasAbility(Box<Ability>),
    AddSubtype(Vec<Subtype>),
    CantAttackOrBlock,
    CantActivateNonManaAbil,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum NumberComputer {
    NumPermanents(Vec<Constraint>),
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum ClauseEffect {
    Destroy,
    Exile,
    AddMana(Vec<ManaCostSymbol>),
    GainLife(i64),
    DrawCard,
    Tap,
    Compound(Vec<Clause>),
    SetTargetController(Box<Clause>),
    CreateToken(Vec<TokenAttribute>),
    UntilEndTurn(ContEffect),
    MultClause(Box<ClauseEffect>, NumberComputer),
    PutCounter(Counter, i64),
}
