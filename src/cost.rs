use crate::mana::{Color, ManaCostSymbol};
use crate::game::Game;
use crate::player::Player;
use anyhow::{bail, Result, Error};
use hecs::Entity;

/*
!!!!!!!!!TODO
Fix this to check that the cost obligations are
fulfilled by the supplied mana. This should enable
hybrid mana with ease 
*/
#[derive(Clone, Debug)]
pub enum Cost {
    Mana(ManaCostSymbol),
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
            &Cost::Mana(symbol)=>{
                true
            },
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

    pub async fn pay(
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
            Cost::Mana(symbol) => {
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
                game.tap(payment).await;
                Ok(payment)
            }
        }
    }
}
