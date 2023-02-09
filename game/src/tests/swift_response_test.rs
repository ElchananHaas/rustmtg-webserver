use anyhow::Result;
use common::zones::Zone;
use test_log;

use crate::{event::EventResult, tests::common_test::test_state_w_decks};

#[test_log::test(tokio::test)]
async fn test_gagglemaster() -> Result<()> {
    let deck = vec!["Swift Response", "Alpine Watchdog"];
    let mut game = test_state_w_decks(deck)?;
    let pl = game.active_player;
    let library = game.players.get(pl).unwrap().library.clone();
    let move_bat = game
        .move_zones(library[1], Zone::Library, Zone::Battlefield)
        .await;
    assert!(move_bat.len() == 1);
    let EventResult::MoveZones { oldent:_, newent, source:_, dest:_ }=move_bat[0] else{
        panic!("failed to move zones");
    };
    let watchdog = newent.unwrap();
    let hand = game.draw(pl).await[0];
    {
        let swift = game.cards.get(hand).unwrap();
        assert!(!swift.effect.iter().any(|x| game.is_valid_target(
            x,
            hand,
            watchdog.into(),
            Zone::Battlefield
        )));
    }
    assert!(game.tap(watchdog).await);
    {
        let swift = game.cards.get(hand).unwrap();
        assert!(swift.effect.iter().any(|x| game.is_valid_target(
            x,
            hand,
            watchdog.into(),
            Zone::Battlefield
        )));
    }
    Ok(())
}
