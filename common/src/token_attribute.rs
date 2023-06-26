use mtg_log_macro::MTGLoggable;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::log::{MTGLog,GameContext};

use crate::{
    ability::Ability,
    card_entities::PT,
    cardtypes::{Subtype, Type},
    mana::Color,
};

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq, MTGLoggable)]
pub enum TokenAttribute {
    PT(PT),
    HasColor(Color),
    Type(Type),
    Subtype(Subtype),
    Ability(Ability),
    EntersTappedAndAttacking,
}
