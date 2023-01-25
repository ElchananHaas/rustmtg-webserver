use crate::card_entities::PT;
use crate::cardtypes::Type;
use crate::entities::PlayerId;
use crate::mana::ManaCostSymbol;
use crate::{entities::TargetId, token_attribute::TokenAttribute};
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
#[derive(Clone, Serialize, JsonSchema, Debug, PartialEq, Eq)]
pub enum ContDuration {
    Perpetual,
    EndOfTurn,
}
#[derive(Clone, Serialize, JsonSchema, Debug)]
pub struct Continuous {
    pub effect: ContEffect,
    pub affected: Affected,
    pub constraints: Vec<ClauseConstraint>,
    pub duration: ContDuration,
    pub controller: PlayerId, //The player controlling the spell, ability or permanent
                              //that generated this continuous effect.
}

#[derive(Clone, Serialize, JsonSchema, Debug)]
pub enum ContEffect {
    ModifyPT(PT),
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
    UntilEndTurn(ContEffect),
}
