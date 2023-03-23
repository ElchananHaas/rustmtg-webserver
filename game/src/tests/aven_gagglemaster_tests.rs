use crate::{event::Event, tests::common_test::test_state};
use anyhow::Result;
use common::zones::Zone;
use test_log;

#[test_log::test(tokio::test)]
async fn test_gagglemaster() -> Result<()> {
    let mut game = test_state()?;
    let pl = game.active_player;
    let mut gagglemasters = Vec::new();
    for i in 0..3 {
        assert!(game.players.get(pl).unwrap().hand.len() == i);
        let drawn = game.draw(pl).await;
        assert!(drawn.len() == 1);
        gagglemasters.push(drawn[0]);
    }
    println!("{:#?}", game.cards.get(gagglemasters[0]));
    for i in 0..3 {
        let start_life = game.players.get(pl).unwrap().life;
        let _moved = game
            .handle_event(Event::MoveZones {
                ent: gagglemasters[i],
                origin: Some(Zone::Hand),
                dest: Zone::Battlefield,
            })
            .await;
        game.resolve(game.stack[0]).await;
        let end_life = game.players.get(pl).unwrap().life;
        assert_eq!(start_life + 2 * (i as i64 + 1), end_life);
    }
    assert!(game.players.get(pl).unwrap().hand.len() == 0);
    assert!(game.battlefield.len() == 3);
    //println!("{:?}",game.triggered_abilities);
    Ok(())
}
