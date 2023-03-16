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
    tests::common_test::hand_battlefield_setup,
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
            ent: acolyte,
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
