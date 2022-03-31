use serde_derive::Serialize;

use crate::cost::Cost;
use crate::spellabil::Clause;
use crate::spellabil::KeywordAbility;
//use crate::carddb::CardBuilder;
//origin entity, target entity

#[derive(Clone, Serialize)]
pub struct TriggeredAbility {}

#[derive(Clone, Serialize)]
pub struct StaticAbility {}
#[derive(Clone, Serialize)]
pub struct ActivatedAbility {
    pub costs: Vec<Cost>,
    pub effect: Vec<Clause>,
    pub keyword: Option<KeywordAbility>,
}
#[derive(Clone, Serialize)]
pub enum Ability {
    Activated(ActivatedAbility),
    Triggered(TriggeredAbility),
    Static(StaticAbility),
}

impl Ability {
    pub fn keyword(&self) -> Option<KeywordAbility> {
        match self {
            Self::Activated(abil) => abil.keyword,
            _ => None,
        }
    }
}
