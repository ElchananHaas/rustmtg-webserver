//A list of a large number of ability/clause creating functions

use anyhow::Result;

use crate::{
    ability::Ability,
    cost::Cost,
    game::Game,
    mana::{mana_cost_string, ManaCostSymbol},
    player::Player,
    spellabil::{ClauseEffect, SpellAbilBuilder},
};

pub fn tap_for_mana(mana: Vec<ManaCostSymbol>) -> Ability {
    let mut builder = SpellAbilBuilder::new();
    builder.clause(ClauseEffect::AddMana(mana));
    builder.activated_ability(vec![Cost::Selftap], true)
}
