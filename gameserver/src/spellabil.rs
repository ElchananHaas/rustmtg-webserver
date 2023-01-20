use crate::{
    card_types::Type,
    entities::TargetId,
    game::Game,
    token_builder::TokenAttribute,
};
use common::mana::{Color, ManaCostSymbol};
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
impl ClauseConstraint {
    pub fn passes_constraint(&self, game: &Game, id: TargetId) -> bool {
        match self{
            ClauseConstraint::IsTapped => {
                if let TargetId::Card(card)=id
                && let Some(ent)=game.cards.get(card){
                    ent.tapped
                }else{
                    false
                }
            },
            ClauseConstraint::CardType(t) => {
                if let TargetId::Card(card)=id
                && let Some(ent)=game.cards.get(card){
                    ent.types.get(*t)
                }else{
                    false
                }                        
            },
            ClauseConstraint::Or(constraints) => {
                for c in constraints{
                    if c.passes_constraint(game, id){
                        return true
                    }
                }
                false
            }
        }
    }
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
