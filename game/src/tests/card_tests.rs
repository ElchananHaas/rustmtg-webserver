use std::num::NonZeroU64;

use anyhow::Result;
use common::{ zones::Zone, counters::Counter, entities::PlayerId};
use test_log;

use crate::tests::common_test::hand_battlefield_setup;

#[test_log::test(tokio::test)]
async fn test_basri_solidarity() -> Result<()> {
    let (mut game, hand) =
        hand_battlefield_setup(vec!["Basri's Solidarity";2], vec!["Wishcoin Crab";2], None).await?;
    let basri_1 = *hand.iter().next().unwrap();
    game.move_zones(basri_1, Zone::Hand, Zone::Stack).await;
    game.resolve(game.stack[0]).await;
    for ent in &game.battlefield{
        if let Some(card)=game.cards.get(*ent){
            assert!(card.counters==vec![Counter::Plus1Plus1]);
        }
    }
    {
        let ent=game.battlefield.iter().next().unwrap();
        if let Some(card)=game.cards.get_mut(*ent){
            card.set_controller(Some(PlayerId::from(NonZeroU64::new(5).unwrap())));
        }
    }
    let basri = *game.players.get(game.active_player).unwrap().hand.iter().next().unwrap();
    game.move_zones(basri, Zone::Hand, Zone::Stack).await;
    game.resolve(game.stack[0]).await;
    let mut total_counters=0;
    for ent in &game.battlefield{
        if let Some(card)=game.cards.get(*ent){
            total_counters+=&card.counters.len();
        }
    }
    assert!(total_counters==3);
    Ok(())
}
