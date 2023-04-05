use std::num::NonZeroU64;

use anyhow::Result;
use common::{counters::Counter, entities::PlayerId, zones::Zone};
use test_log;

use crate::tests::common_test::{get_hand_card, hand_battlefield_setup};

#[test_log::test(tokio::test)]
async fn test_containment_priest() -> Result<()> {
    let (mut game, _hand) = hand_battlefield_setup(
        vec!["Wishcoin Crab"; 3],
        vec!["Containment Priest"; 1],
        None,
    )
    .await?;
    assert!(game.battlefield.len() == 1);
    let card = get_hand_card(&game);
    game.move_zones(vec![card], Zone::Hand, Zone::Battlefield)
        .await;
    assert!(game.battlefield.len() == 1);
    let card = get_hand_card(&game);
    game.move_zones(vec![card], Zone::Hand, Zone::Stack).await;
    {
        game.cards.get_mut(game.stack[0]).unwrap().cast = true;
    }
    game.resolve(game.stack[0]).await;
    assert!(game.battlefield.len() == 2);
    Ok(())
}
#[test_log::test(tokio::test)]
async fn test_basri_solidarity() -> Result<()> {
    let (mut game, hand) = hand_battlefield_setup(
        vec!["Basri's Solidarity"; 2],
        vec!["Wishcoin Crab"; 2],
        None,
    )
    .await?;
    let basri_1 = *hand.iter().next().unwrap();
    game.move_zones(vec![basri_1], Zone::Hand, Zone::Stack)
        .await;
    game.resolve(game.stack[0]).await;
    for ent in &game.battlefield {
        if let Some(card) = game.cards.get(*ent) {
            assert!(card.counters == vec![Counter::Plus1Plus1]);
        }
    }
    {
        let ent = game.battlefield.iter().next().unwrap();
        if let Some(card) = game.cards.get_mut(*ent) {
            card.set_controller(Some(PlayerId::from(NonZeroU64::new(5).unwrap())));
        }
    }
    let basri = *game
        .players
        .get(game.active_player)
        .unwrap()
        .hand
        .iter()
        .next()
        .unwrap();
    game.move_zones(vec![basri], Zone::Hand, Zone::Stack).await;
    game.resolve(game.stack[0]).await;
    let mut total_counters = 0;
    for ent in &game.battlefield {
        if let Some(card) = game.cards.get(*ent) {
            total_counters += &card.counters.len();
        }
    }
    assert!(total_counters == 3);
    Ok(())
}
