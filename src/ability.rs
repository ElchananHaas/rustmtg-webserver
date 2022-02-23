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
pub struct Ability {
    pub mana_ability: bool,
    pub keyword: Option<KeywordAbility>,
    pub abil: AbilityType,
}
#[derive(Clone, Serialize)]
pub enum AbilityType {
    Activated {
        cost: Vec<Cost>,
        effect: Vec<Clause>,
    },
    Triggered(TriggeredAbility),
    Static(StaticAbility),
}