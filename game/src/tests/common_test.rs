use std::collections::HashMap;

use crate::{
    game::{build_game::GameBuilder, Game},
    player::{MockClient, PlayerCon, TestClient},
};
use anyhow::Result;
use carddb::carddb::CardDB;
use common::{entities::CardId, hashset_obj::HashSetObj, zones::Zone};

fn get_db() -> &'static CardDB {
    crate::CARDDB.get_or_init(|| CardDB::new())
}
pub fn test_state_w_decks(deck: Vec<&'static str>) -> Result<Game> {
    let db: &CardDB = get_db();
    let mut gamebuild = GameBuilder::new();
    gamebuild.add_player("p1", &db, &deck, PlayerCon::new_test(TestClient::default()))?;
    gamebuild.add_player("p2", &db, &deck, PlayerCon::new_test(TestClient::default()))?;
    gamebuild.build(&db)
}

pub fn test_state() -> Result<Game> {
    let mut deck = Vec::new();
    for _ in 1..(60 - deck.len()) {
        deck.push("Aven Gagglemaster");
    }
    test_state_w_decks(deck)
}
pub async fn hand_battlefield_setup(
    hand: Vec<&'static str>,
    battlefield: Vec<&'static str>,
    active_client: Option<Box<dyn MockClient>>,
) -> Result<(Game, HashSetObj<CardId>)> {
    let joined: Vec<&'static str> = hand
        .iter()
        .cloned()
        .chain(battlefield.iter().cloned())
        .collect();
    let db: &CardDB = get_db();
    let mut gamebuild = GameBuilder::new();
    let active_client =
        active_client.map_or_else(|| TestClient::default(), |x| TestClient::with_client(x));
    gamebuild.add_player("p1", &db, &joined, PlayerCon::new_test(active_client))?;
    gamebuild.add_player(
        "p2",
        &db,
        &joined,
        PlayerCon::new_test(TestClient::default()),
    )?;
    let mut game = gamebuild.build(&db)?;
    let pl = game.active_player;
    let mut library = game.players.get(pl).unwrap().library.clone();
    for _ in 0..battlefield.len() {
        let top = library.pop().unwrap();
        let move_bat = game
            .move_zones(vec![top], Zone::Library, Zone::Battlefield)
            .await;
        assert!(move_bat.len() == 1);
    }
    for _ in 0..library.len() {
        game.draw(pl).await;
    }
    let pl_hand = game.players.get(pl).unwrap().hand.clone();
    assert!(pl_hand.len() == hand.len());
    game.send_state().await;
    game.layers_state_actions().await;
    game.send_state().await;
    Ok((game, pl_hand))
}
pub fn cards_with_name(state: &Game, name: &str) -> Vec<CardId> {
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

pub fn by_name(state: &Game) -> HashMap<String, Vec<CardId>> {
    let mut res: HashMap<String, Vec<CardId>> = HashMap::new();
    for cardid in &state.battlefield {
        if let Some(card) = state.cards.get(*cardid) {
            res.entry(card.name.to_owned()).or_default().push(*cardid);
        }
    }
    res
}

#[test_log::test]
fn test_game_init() -> Result<()> {
    let db = get_db();
    let mut gamebuild = GameBuilder::new();
    let mut deck = Vec::new();
    deck.push("Staunch Shieldmate");
    deck.push("Garruk's Gorehorn");
    deck.push("Alpine Watchdog");
    //deck.push("Mistral Singer");
    deck.push("Wishcoin Crab");
    deck.push("Blood Glutton");
    deck.push("Walking Corpse");
    deck.push("Onakke Ogre");
    deck.push("Colossal Dreadmaw");
    deck.push("Concordia Pegasus");
    for _ in 1..(60 - deck.len()) {
        deck.push("Plains");
    }
    gamebuild.add_player("p1", &db, &deck, PlayerCon::new_test(TestClient::default()))?;
    gamebuild.add_player("p2", &db, &deck, PlayerCon::new_test(TestClient::default()))?;
    let _game = gamebuild.build(&db);
    Ok(())
}
