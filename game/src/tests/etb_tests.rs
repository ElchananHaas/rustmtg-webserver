use std::{num::NonZeroU64};

use anyhow::Result;
use common::{entities::{CardId, TargetId}, hashset_obj::HashSetObj, zones::Zone};
use serde::{Serialize, Deserialize};
use test_log;

use crate::{tests::common_test::hand_battlefield_setup, player::MockClient, client_message::{GameState, AskSelectN}, event::Event};

struct AcolyteClient{}

impl MockClient for AcolyteClient{
    fn select_targets(&self, _game: &GameState, ask: &AskSelectN<TargetId>) -> HashSetObj<usize> {
        let mut res=HashSetObj::new();
        assert!(ask.min==0);
        assert!(ask.max==2);
        assert!(ask.ents.len()==2);
        for (a,_) in ask.ents.iter().take(2).enumerate(){
            res.add(a);
        }
        res
    }
}
#[test_log::test(tokio::test)]
async fn basris_acolyte_test() -> Result<()> {
    let (mut game, hand) =
        hand_battlefield_setup(vec!["Basri's Acolyte"], vec!["Staunch Shieldmate"; 2], Some(Box::new(AcolyteClient{})))
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
    assert!(game.battlefield.len()==3);
    let mut coutner_count=0;
    for ent in game.battlefield{
        if let Some(card)=game.cards.get(ent){
            coutner_count+=card.counters.len();
        }
    }
    assert!(coutner_count==2);
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
#[derive(Serialize,Deserialize)]
struct Test{
    #[serde(flatten)]
    inner:std::collections::HashSet<CardId>
}
#[test]
fn serde_test() -> Result<()> {
    let mut test=Test{
        inner:std::collections::HashSet::new()
    };
    test.inner.insert(CardId::from(NonZeroU64::new(500).unwrap()));
    test.inner.insert(CardId::from(NonZeroU64::new(365).unwrap()));
    let mut buffer = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buffer);
        let mut json_serial = serde_json::Serializer::new(cursor);
        test.serialize(&mut json_serial)
            .expect("serialized correctly");
    }
    let contents = std::str::from_utf8(&buffer).expect("json is valid text");
    dbg!(contents);
    let _remade: Test = serde_json::from_str(contents).expect("parsed correctly");
    Ok(())
}
