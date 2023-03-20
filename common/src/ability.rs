use schemars::JsonSchema;
use serde::Deserialize;
use serde_derive::Serialize;

use crate::cost::Cost;
use crate::entities::CardId;
use crate::mana::ManaCostSymbol;
use crate::spellabil::Clause;
use crate::spellabil::ClauseEffect;
use crate::spellabil::KeywordAbility;
use crate::spellabil::{Affected, PermConstraint};
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
    pub constraint: Vec<PermConstraint>,
    pub trigger: AbilityTriggerType,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct TriggeredAbility {
    pub trigger: AbilityTrigger,
    pub effect: Vec<Clause>,
    pub keyword: Option<KeywordAbility>,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum PreventionEffect {
    Unused, //Needed becuase typescript can't handle empty enums
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct ContPrevention {
    pub source: CardId,
    pub effect: PreventionEffect,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub enum StaticAbilityEffect {
    GivenByKeyword,
    Protection(PermConstraint),
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
}
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
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
            Self::Static(abil) => abil.keyword,
            Self::Triggered(abil) => abil.keyword,
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
