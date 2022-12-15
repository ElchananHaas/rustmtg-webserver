use schemars::JsonSchema;
use serde_derive::Serialize;

use crate::cost::Cost;
use crate::mana::Color;
use crate::mana::ManaCostSymbol;
use crate::spellabil::Affected;
use crate::spellabil::Clause;
use crate::spellabil::ClauseEffect;
use crate::spellabil::KeywordAbility;
//use crate::carddb::CardBuilder;
//origin entity, target entity

#[derive(Clone, Serialize, JsonSchema, Debug)]
pub struct TriggeredAbility {}

#[derive(Clone, Serialize, JsonSchema, Debug)]
pub enum StaticAbilityEffect {
    HasColor(Color),
    GivenByKeyword,
}
#[derive(Clone, Serialize, JsonSchema, Debug)]
pub struct StaticAbility {
    pub keyword: Option<KeywordAbility>,
    pub effect: StaticAbilityEffect,
}
#[derive(Clone, Serialize, JsonSchema, Debug)]
pub struct ActivatedAbility {
    pub costs: Vec<Cost>,
    pub effect: Vec<Clause>,
    pub keyword: Option<KeywordAbility>,
}
#[derive(Clone, Serialize, JsonSchema, Debug)]
pub enum Ability {
    Activated(ActivatedAbility),
    Triggered(TriggeredAbility),
    Static(StaticAbility),
}

impl Ability {
    pub fn keyword(&self) -> Option<KeywordAbility> {
        match self {
            //TODO! add in other types of keywords as I add them
            Self::Activated(abil) => abil.keyword,
            _ => None,
        }
    }
}

impl Ability {
    pub fn tap_for_mana(mana: Vec<ManaCostSymbol>) -> Self {
        Ability::Activated(ActivatedAbility {
            costs: vec![Cost::Selftap],
            effect: vec![Clause {
                effect: ClauseEffect::AddMana(mana),
                constraints: Vec::new(),
                affected: Affected::Controller,
            }],
            keyword: None,
        })
    }
    pub fn from_keyword(keyword: KeywordAbility) -> Self {
        Ability::Static(StaticAbility {
            keyword: Some(keyword),
            effect: StaticAbilityEffect::GivenByKeyword,
        })
    }
}
