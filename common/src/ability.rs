use schemars::JsonSchema;
use serde::Deserialize;
use serde_derive::Serialize;

use crate::cost::Cost;
use crate::mana::ManaCostSymbol;
use crate::spellabil::Clause;
use crate::spellabil::ClauseEffect;
use crate::spellabil::KeywordAbility;
use crate::spellabil::{Affected, Constraint};
use crate::zones::Zone;
//use crate::carddb::CardBuilder;
//origin entity, target entity

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct ZoneMoveTrigger {
    //These both must match for the ability to trigger
    pub origin: Option<Zone>,
    pub dest: Option<Zone>,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum AbilityTriggerType {
    ZoneMove(ZoneMoveTrigger),
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct AbilityTrigger {
    pub constraint: Vec<Constraint>,
    pub trigger: AbilityTriggerType,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct TriggeredAbility {
    pub trigger: AbilityTrigger,
    pub effect: Vec<Clause>,
    pub keyword: Option<KeywordAbility>,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum StaticAbilityEffect {
    GivenByKeyword,
    Protection(Constraint),
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct StaticAbility {
    pub keyword: Option<KeywordAbility>,
    pub effect: StaticAbilityEffect,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct ActivatedAbility {
    pub costs: Vec<Cost>,
    pub effect: Vec<Clause>,
    pub keyword: Option<KeywordAbility>,
    pub restrictions: Option<Constraint>,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum Replacement {
    ZoneMoveReplacement {
        constraints: Vec<Constraint>,
        trigger: ZoneMoveTrigger,
        new_effect: Clause,
    },
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct ReplacementAbility {
    pub keyword: Option<KeywordAbility>,
    pub effect: Replacement,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum Ability {
    Activated(ActivatedAbility),
    Triggered(TriggeredAbility),
    Static(StaticAbility),
    Replacement(ReplacementAbility),
}

impl Ability {
    pub fn keyword(&self) -> Option<KeywordAbility> {
        match self {
            //TODO! add in other types of keywords as I add them
            Self::Activated(abil) => abil.keyword,
            Self::Static(abil) => abil.keyword,
            Self::Triggered(abil) => abil.keyword,
            Self::Replacement(abil) => abil.keyword,
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
            restrictions: None,
        })
    }
    pub fn from_keyword(keyword: KeywordAbility) -> Self {
        Ability::Static(StaticAbility {
            keyword: Some(keyword),
            effect: StaticAbilityEffect::GivenByKeyword,
        })
    }
}
