use anyhow::Result;
use common::{cardtypes::Subtype, zones::Zone};
use test_log;

use crate::tests::common_test::hand_battlefield_setup;

#[test_log::test(tokio::test)]
async fn test_baneslayer() -> Result<()> {
    let (mut game, hand) =
        hand_battlefield_setup(vec!["Murder"], vec!["Baneslayer Angel"], None).await?;
    let baneslayer = *game.battlefield.iter().next().unwrap();
    let murder_id = *hand.iter().next().unwrap();
    {
        let murder = game.cards.get(murder_id).unwrap();
        assert!(murder.effect.iter().any(|x| game.is_valid_target(
            &x.constraints,
            murder_id,
            baneslayer.into(),
            Zone::Battlefield
        )));
    }
    {
        let murder = game.cards.get_mut(murder_id).unwrap();
        murder.subtypes.add(Subtype::Demon);
    }
    {
        let murder = game.cards.get(murder_id).unwrap();
        assert!(!murder.effect.iter().any(|x| game.is_valid_target(
            &x.constraints,
            murder_id,
            baneslayer.into(),
            Zone::Battlefield
        )));
    }

    Ok(())
}
