use cardtypes::{Subtype, Type};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{ability::Ability, card_entities::PT, mana::Color};

#[derive(Clone, Serialize, JsonSchema, Debug)]
pub enum TokenAttribute {
    PT(PT),
    HasColor(Color),
    Type(Type),
    Subtype(Subtype),
    Ability(Ability),
}
