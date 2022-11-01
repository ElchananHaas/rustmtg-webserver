use schemars::JsonSchema;
use serde_derive::Serialize;

use crate::cost::Cost;
use crate::mana::ManaCostSymbol;
use crate::spellabil::Clause;
use crate::spellabil::KeywordAbility;
//use crate::carddb::CardBuilder;
//origin entity, target entity

#[derive(Clone, Serialize, JsonSchema)]
pub struct TriggeredAbility {}

#[derive(Clone, Serialize, JsonSchema)]
pub struct StaticAbility {
    pub keyword: Option<KeywordAbility>
}
#[derive(Clone, Serialize, JsonSchema)]
pub struct ActivatedAbility {
    pub costs: Vec<Cost>,
    pub effect: Vec<Clause>,
    pub keyword: Option<KeywordAbility>,
}
#[derive(Clone, Serialize, JsonSchema)]
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
            effect: vec![Clause::AddMana(mana)],
            keyword: None,
        })
    }
}
