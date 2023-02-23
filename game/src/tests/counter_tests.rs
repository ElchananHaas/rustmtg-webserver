use anyhow::Result;
use common::{counters::Counter, card_entities::PT};
use test_log;

use crate::tests::common_test::hand_battlefield_setup;

#[test_log::test(tokio::test)]
async fn test_plus1plus1_counter() -> Result<()> {
    let (mut game, _hand) = hand_battlefield_setup(vec![], vec!["Staunch Shieldmate"]).await?;
    let creature = *game.battlefield.iter().next().unwrap();
    {
        let c=game.cards.get_mut(creature).unwrap();
        assert!(*c.pt.as_ref().unwrap()==PT{
            power:1,
            toughness:3,
        });
        c.counters.push(Counter::Plus1Plus1);
    }
    game.layers_state_actions().await;
    {
        let c=game.cards.get(creature).unwrap();
        assert!(*c.pt.as_ref().unwrap()==PT{
            power:2,
            toughness:4,
        });

    }
    Ok(())
}
