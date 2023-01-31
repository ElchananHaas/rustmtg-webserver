use crate::{
    event::Event,
    game::{build_game::GameBuilder, Game},
    player::PlayerCon,
};
use anyhow::Result;
use carddb::carddb::CardDB;
use common::zones::Zone;
use test_log;
fn get_db() -> &'static CardDB {
    crate::CARDDB.get_or_init(|| CardDB::new())
}
fn test_state() -> Result<Game> {
    let db: &CardDB = get_db();
    let mut gamebuild = GameBuilder::new();
    let mut deck = Vec::new();
    for _ in 1..(60 - deck.len()) {
        deck.push("Aven Gagglemaster");
    }
    gamebuild.add_player("p1", &db, &deck, PlayerCon::new_test())?;
    gamebuild.add_player("p2", &db, &deck, PlayerCon::new_test())?;
    gamebuild.build(&db)
}

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
    //println!("{:?}",game.cards.get(gagglemasters[0]));
    for i in 0..3 {
        let start_life=game.players.get(pl).unwrap().life;
        let _moved = game
            .handle_event(Event::MoveZones {
                ent: gagglemasters[i],
                origin: Some(Zone::Hand),
                dest: Zone::Battlefield,
            })
            .await;
        game.resolve(game.stack[0]).await;
        game.stack.pop();
        let end_life=game.players.get(pl).unwrap().life;
        assert_eq!(start_life+2*(i as i64 +1),end_life);
    }
    assert!(game.players.get(pl).unwrap().hand.len() == 0);
    assert!(game.battlefield.len() == 3);
    //println!("{:?}",game.triggered_abilities);
    Ok(())
}
