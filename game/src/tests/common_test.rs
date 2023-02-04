use carddb::carddb::CardDB;
use common::{entities::CardId, zones::Zone, hashset_obj::HashSetObj};
use crate::{
    game::{build_game::GameBuilder, Game},
    player::PlayerCon,
};
use anyhow::Result;

fn get_db() -> &'static CardDB {
    crate::CARDDB.get_or_init(|| CardDB::new())
}
pub fn test_state_w_decks(deck:Vec<&'static str>) -> Result<Game>{
    let db: &CardDB = get_db();
    let mut gamebuild = GameBuilder::new();
    gamebuild.add_player("p1", &db, &deck, PlayerCon::new_test())?;
    gamebuild.add_player("p2", &db, &deck, PlayerCon::new_test())?;
    gamebuild.build(&db)
}
pub fn test_state() -> Result<Game> {
    let mut deck = Vec::new();
    for _ in 1..(60 - deck.len()) {
        deck.push("Aven Gagglemaster");
    }
    test_state_w_decks(deck)
}
pub async fn hand_battlefield_setup(hand:Vec<&'static str>,battlefield:Vec<&'static str>) -> Result<(Game,HashSetObj<CardId>)>{
    let joined:Vec<&'static str>=hand.iter().cloned().chain(battlefield.iter().cloned()).collect();
    let mut game=test_state_w_decks(joined)?;
    let pl = game.active_player;
    let mut library=game.players.get(pl).unwrap().library.clone();
    for _ in 0..battlefield.len(){
        let top=library.pop().unwrap();
        let move_bat=game.move_zones(top, Zone::Library, Zone::Battlefield).await;
        assert!(move_bat.len()==1);
    }
    for _ in 0..library.len(){
        game.draw(pl).await;
    }
    let pl_hand=game.players.get(pl).unwrap().hand.clone();
    assert!(pl_hand.len()==hand.len());
    game.layers_state_actions().await;
    Ok((game,pl_hand))
}
pub fn cards_with_name(state: &mut Game, name: &str) -> Vec<CardId> {
    state
        .cards_and_zones()
        .iter()
        .filter_map(|(id, _zone)| {
            state.cards.get(*id).map_or(
                None,
                |card| {
                    if card.name == name {
                        Some(*id)
                    } else {
                        None
                    }
                },
            )
        })
        .collect()
}