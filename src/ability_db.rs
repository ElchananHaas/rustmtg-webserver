//A list of a large number of ability/clause creating functions

use std::sync::Arc;
use anyhow::{ Result};
use hecs::Entity;

use crate::{ability::Ability, spellabil::SpellAbilBuilder, player::Player, game::Game, cost::Cost};

fn tap_for_mana()->Ability{
    let mut builder = SpellAbilBuilder::new();
    builder.clause(Arc::new(|game:&mut Game, ent_n:Entity| {
        let _:Result<()>=try{
            let controller=game.get_controller(ent_n)?;
            let pl=game.ents.get_mut::<Player>(controller)?;
            game.tap(ent_n).await;
            //TODO create mana!
        };
    }));
    builder.activated_ability(Cost::Selftap,true)
}