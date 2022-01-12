use std::fmt;
//use crate::carddb::CardBuilder;
use crate::carddb::Cardbuildtype;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeywordAbility {
    FirstStrike,
    Flying,
    Haste,
    Vigilance,
}

#[derive(Clone, Debug)]
pub struct TriggeredAbility {}
#[derive(Clone)]
pub struct ActivatedAbility {
    mana_ability: bool,
    effect: Cardbuildtype,
}
impl fmt::Debug for ActivatedAbility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActivatedAbility")
            .field("mana_ability", &self.mana_ability)
            .finish()
    }
}
#[derive(Clone, Debug)]
pub struct StaticAbility {}
#[derive(Clone, Debug)]
pub enum Ability {
    Activated(ActivatedAbility),
    Triggered(TriggeredAbility),
    Static(StaticAbility),
}
impl Ability {
    pub fn keyword(&self) -> Option<KeywordAbility> {
        todo!("Can't tell if ability is keyword yet!");
    }
}
