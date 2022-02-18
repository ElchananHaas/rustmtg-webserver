//A list of a large number of ability/clause creating functions

use std::sync::Arc;
use anyhow::{ Result};
use hecs::Entity;

use crate::{ability::Ability, spellabil::{SpellAbilBuilder, ClauseEffect}, player::Player, game::Game, cost::Cost, mana::{mana_cost_string, ManaCostSymbol}};

pub fn tap_for_mana(mana:Vec<ManaCostSymbol>)->Ability{
    let mut builder = SpellAbilBuilder::new();
    builder.clause(ClauseEffect::AddMana(mana));
    builder.activated_ability(vec![Cost::Selftap],true)
}