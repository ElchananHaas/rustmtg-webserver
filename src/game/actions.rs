use serde::Serialize;
use serde_derive::Serialize;

use crate::{
    ability::Ability,
    cost::Cost,
    entities::{CardId, PlayerId},
    player::Player,
    spellabil::KeywordAbility,
};

use super::Zone;

//Checks to see if casting option's rules were followed
//The zone it can be cast from will implicitly be enabled by
//the code generating casting options
#[derive(Clone, Serialize)]
pub enum ActionFilter {
    None,
}
impl ActionFilter {
    pub fn check(&self) -> bool {
        match self {
            ActionFilter::None => true,
            _ => todo!(),
        }
    }
}
#[derive(Clone, Serialize)]
pub struct CastingOption {
    pub card: CardId,
    pub zone: Zone,
    pub costs: Vec<Cost>,
    pub filter: ActionFilter,
    pub keyword: Option<KeywordAbility>,
    pub player: PlayerId,
}
#[derive(Clone, Serialize)]
pub struct AbilityOption {
    pub source: CardId,
    pub index: usize,
}
//Every action the player can take.
#[derive(Clone, Serialize)]
pub enum Action {
    Cast(CastingOption),
    PlayLand(CardId),
    ActivateAbility { source: CardId, index: usize },
}
