use crate::event::EventCause;
use crate::game::Color;
use crate::game::Game;
use crate::player::Player;
use anyhow::{bail, Result};
use hecs::Entity;

#[derive(Clone, Debug)]
pub enum Cost {
    Generic,
    Color(Color),
    Selftap,
}

impl Cost {
    //Determines if an entity can be used towards a payment
    //mana could be used towards the payment for it, so this would return true
    //This will take into account any prevention effects, so
    //it can be relied upon to be correct.

    //For a mana cost, the payment is the mana entity used to pay the cost

    //For a selftap, the payment is the entity that is tapping itself.
    pub fn valid_payment(
        &self,
        game: &Game,
        source: Entity,
        controller: Entity,
        payment: Entity,
    ) -> bool {
        match self {
            Cost::Generic => {
                if let Ok(player) = game.ents.get::<Player>(controller) {
                    //Handle prevention effects
                    player.mana_pool.contains(&payment)
                } else {
                    false
                }
            }
            Cost::Color(color) => {
                if let Ok(player) = game.ents.get::<Player>(controller) {
                    //Handle prevention effects
                    if let Some(mana) = player.mana_pool.get(&payment) {
                        if let Ok(poolcolor) = game.ents.get::<Color>(*mana) {
                            //Handle prevention effects/restrictions here!
                            *color == *poolcolor
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Cost::Selftap => {
                //Similarly handle prevention effects here!
                if source == payment {
                    game.can_tap(payment)
                } else {
                    false
                }
            }
        }
    }

    pub fn pay(
        &self,
        game: &mut Game,
        source: Entity,
        controller: Entity,
        payment: Entity,
    ) -> Result<Entity> {
        if !self.valid_payment(game, source, controller, payment) {
            bail!("Invalid payment!");
        }
        match self {
            Cost::Generic | Cost::Color(_) => {
                if let Ok(mut player) = game.ents.get_mut::<Player>(controller) {
                    //TODO Handle prevention effects/restrictions here!
                    if player.mana_pool.remove(&payment) {
                        Ok(payment)
                    } else {
                        bail!("Mana not present in pool!")
                    }
                } else {
                    bail!("Player is gone!")
                }
            }
            Cost::Selftap => {
                //Similarly handle prevention effects here!
                game.tap(payment, EventCause::None);
                Ok(payment)
            }
        }
    }
}
