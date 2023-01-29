use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    ability::Ability,
    card_entities::PT,
    cardtypes::{Subtype, Type},
    mana::Color,
};

#[derive(Clone, Serialize, JsonSchema, Debug, PartialEq)]
pub enum TokenAttribute {
    PT(PT),
    HasColor(Color),
    Type(Type),
    Subtype(Subtype),
    Ability(Ability),
}
