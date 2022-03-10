use serde::Serialize;
use serde_derive::Serialize;

use crate::{cost::Cost, entities::CardId, spellabil::KeywordAbility};

use super::Zone;

//Checks to see if casting option's rules were followed
//The zone it can be cast from will implicitly be enabled by
//the code generating casting options
#[derive(Clone, Serialize)]
pub struct CastFilter {}
#[derive(Clone, Serialize)]
pub struct CastingOption {
    card: CardId,
    costs: Vec<Cost>,
    filter: CastFilter,
    keyword: Option<KeywordAbility>,
}
pub struct AbilityOption {
    source: CardId,
}
//Every action the player can take.
#[derive(Clone, Serialize)]
pub enum Action {
    Cast(CastingOption),
    PlayLand(CardId),
}
