use std::num::NonZeroU64;

use anyhow::Result;
use common::{
    entities::{CardId, TargetId},
    hashset_obj::HashSetObj,
    zones::Zone,
};
use test_log;

use crate::{
    client_message::{AskSelectN, GameState},
    event::Event,
    player::MockClient,
    tests::common_test::{by_name, cards_with_name, hand_battlefield_setup},
};

struct AcolyteClient {}

impl MockClient for AcolyteClient {
    fn select_targets(
        &mut self,
        _game: &GameState,
        ask: &AskSelectN<TargetId>,
    ) -> HashSetObj<usize> {
        let mut res = HashSetObj::new();
        assert!(ask.min == 0);
        assert!(ask.max == 2);
        assert!(ask.ents.len() == 2);
        for (a, _) in ask.ents.iter().take(2).enumerate() {
            res.add(a);
        }
        res
    }
}
#[test_log::test(tokio::test)]
async fn basris_acolyte_test() -> Result<()> {
    let (mut game, hand) = hand_battlefield_setup(
        vec!["Basri's Acolyte"],
        vec!["Staunch Shieldmate"; 2],
        Some(Box::new(AcolyteClient {})),
    )
    .await?;
    let acolyte = *hand.iter().next().unwrap();
    let _moved = game
        .handle_event(Event::MoveZones {
            ents: vec![acolyte],
            origin: Some(Zone::Hand),
            dest: Zone::Battlefield,
        })
        .await;
    game.cycle_priority().await;
    assert!(game.battlefield.len() == 3);
    let mut coutner_count = 0;
    for ent in game.battlefield {
        if let Some(card) = game.cards.get(ent) {
            coutner_count += card.counters.len();
        }
    }
    assert!(coutner_count == 2);
    Ok(())
}

struct BasriLieutenantClient {
    place_basri: bool,
}

impl MockClient for BasriLieutenantClient {
    fn select_targets(
        &mut self,
        game: &GameState,
        ask: &AskSelectN<TargetId>,
    ) -> HashSetObj<usize> {
        assert!(ask.max == 1 && ask.max == 1);
        for (i, card) in ask.ents.iter().enumerate() {
            let TargetId::Card(cardid)=card else {panic!();};
            let card = game.cards.get(cardid).unwrap();
            if (card.name == "Staunch Shieldmate" && !self.place_basri)
                || (card.name == "Basri's Lieutenant" && self.place_basri)
            {
                let mut res = HashSetObj::new();
                res.add(i);
                return res;
            }
        }
        panic!();
    }
}
#[test_log::test(tokio::test)]
async fn basris_lt_test() -> Result<()> {
    let (mut game, hand) = hand_battlefield_setup(
        vec!["Basri's Lieutenant"],
        vec!["Staunch Shieldmate"; 1],
        Some(Box::new(BasriLieutenantClient { place_basri: false })),
    )
    .await?;
    let lt = *hand.iter().next().unwrap();
    let _moved = game
        .handle_event(Event::MoveZones {
            ents: vec![lt],
            origin: Some(Zone::Hand),
            dest: Zone::Battlefield,
        })
        .await;
    game.cycle_priority().await;
    let staunch = cards_with_name(&game, "Staunch Shieldmate")[0];
    game.destroy(vec![staunch]).await;
    assert!(game.battlefield.len() == 1);
    game.cycle_priority().await;
    assert!(game.battlefield.len() == 2);
    let battlefield = by_name(&game);
    assert!(battlefield.contains_key("Basri's Lieutenant"));
    assert!(battlefield.contains_key("Knight"));
    Ok(())
}

#[test_log::test(tokio::test)]
async fn basris_lt_self_test() -> Result<()> {
    let (mut game, hand) = hand_battlefield_setup(
        vec!["Basri's Lieutenant"],
        vec![],
        Some(Box::new(BasriLieutenantClient { place_basri: true })),
    )
    .await?;
    let lt = *hand.iter().next().unwrap();
    let _moved = game
        .handle_event(Event::MoveZones {
            ents: vec![lt],
            origin: Some(Zone::Hand),
            dest: Zone::Battlefield,
        })
        .await;
    game.cycle_priority().await;
    let basri = game.battlefield.iter().next().unwrap();
    game.destroy(vec![*basri]).await;
    assert!(game.battlefield.len() == 0);
    game.cycle_priority().await;
    assert!(game.battlefield.len() == 1);
    let battlefield = by_name(&game);
    assert!(battlefield.contains_key("Knight"));
    Ok(())
}

#[test_log::test(tokio::test)]
async fn empty_test() -> Result<()> {
    let (mut game, _hand) = hand_battlefield_setup(vec![], vec![], None).await?;
    game.cycle_priority().await;
    Ok(())
}

#[test_log::test(tokio::test)]
async fn single_test() -> Result<()> {
    let (mut game, _hand) = hand_battlefield_setup(vec![], vec![], None).await?;
    game.command
        .add(CardId::from(NonZeroU64::new(500).unwrap()));
    //Where is this bug coming from?
    game.cycle_priority().await;
    Ok(())
}