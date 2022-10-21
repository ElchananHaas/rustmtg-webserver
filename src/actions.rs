use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    ability::Ability,
    cost::Cost,
    entities::{CardId, PlayerId},
    player::Player,
    spellabil::KeywordAbility,
};

use crate::game::Zone;

//Checks to see if casting option's rules were followed
//The zone it can be cast from will implicitly be enabled by
//the code generating casting options
#[derive(Clone, Serialize, Debug, JsonSchema)]
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
#[derive(Clone, Serialize, Debug, JsonSchema)]
pub struct CastingOption {
    pub source_card: CardId,
    pub zone: Zone,
    pub costs: Vec<Cost>,
    pub filter: ActionFilter,
    pub player: PlayerId,
}
#[derive(Clone, Serialize, Debug)]
pub struct StackActionOption {
    pub stack_ent: CardId,
    pub ability_source: Option<CardId>,
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
#[derive(Clone, Serialize, JsonSchema, Debug)]
pub enum Action {
    Cast(CastingOption),
    PlayLand(CardId),
    ActivateAbility { source: CardId, index: usize },
}
