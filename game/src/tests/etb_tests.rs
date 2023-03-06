use anyhow::Result;
use common::{ zones::Zone};
use test_log;

use crate::{tests::common_test::hand_battlefield_setup, event::Event};

#[test_log::test(tokio::test)]
async fn etb_tests() -> Result<()> {
    let (mut game, hand) = hand_battlefield_setup(vec!["Basri's Acolyte"], vec!["Staunch Shieldmate";3]).await?;
    let acolyte = *hand.iter().next().unwrap();
    let _moved = game
    .handle_event(Event::MoveZones {
        ent: acolyte,
        origin: Some(Zone::Hand),
        dest: Zone::Battlefield,
    })
    .await;
    dbg!(game.cards.get(game.stack[0]));
    game.resolve(game.stack[0]).await;
    game.stack.pop();
    Ok(())
}
