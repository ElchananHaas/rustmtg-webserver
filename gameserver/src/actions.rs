use schemars::JsonSchema;
use serde::Serialize;

use common::{
    cost::Cost,
    entities::{CardId, PlayerId},
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
    pub possible_to_take: bool, //If the player doesn't have enough resources to
                                //case the spell, this will be false. The user will still have the
                                //option to put it on the stack as per the game rules.
                                //The engine will have a best effort at making this correct for UI benefits
                                //There is a bigger issue if a spell is caatable and the engine thinks it isn't
                                //As opposed to the other way around.
}
#[derive(Clone, Serialize, Debug)]
pub struct StackActionOption {
    pub stack_ent: CardId,
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
