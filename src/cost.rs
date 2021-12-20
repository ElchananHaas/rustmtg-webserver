use crate::game::Game;
use crate::game::{Color, ManaPool};
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
                match game.ents.get::<ManaPool>(controller) {
                    //TODO Handle prevention effects/restrictions here!
                    Ok(pool) => (pool.0.contains(&payment)),
                    _ => false,
                }
            }
            Cost::Color(color) => {
                match game.ents.get::<ManaPool>(controller) {
                    Ok(pool) => {
                        if let Some(mana) = pool.0.get(&payment) {
                            if let Ok(poolcolor) = game.ents.get::<Color>(*mana) {
                                //Handle prevention effects/restrictions here!
                                *color == *poolcolor
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
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
                match game.ents.get::<ManaPool>(controller) {
                    //TODO Handle prevention effects/restrictions here!
                    Ok(pool) => {
                        if pool.0.remove(&payment) {
                            Ok(payment)
                        } else {
                            bail!("Mana not present in pool!")
                        }
                    }
                    _ => {
                        bail!("Player is gone!")
                    }
                }
            }
            Cost::Selftap => {
                //Similarly handle prevention effects here!
                game.tap(payment);
                Ok(payment)
            }
        }
    }
}
