use crate::spellabil::Clause;
use crate::spellabil::KeywordAbility;
//use crate::carddb::CardBuilder;
//origin entity, target entity

#[derive(Debug)]
pub struct TriggeredAbility {}

#[derive(Clone, Debug)]
pub struct StaticAbility {}
pub struct Ability {
    pub mana_ability: bool,
    pub keyword: Option<KeywordAbility>,
    pub abil: AbilityType,
}
pub enum AbilityType{
    Activated{
        effect:Vec<Clause>
    },
    Triggered(TriggeredAbility),
    Static(StaticAbility),
}
impl Ability {
    pub fn keyword(&self) -> Option<KeywordAbility> {
        self.keyword
    }
}