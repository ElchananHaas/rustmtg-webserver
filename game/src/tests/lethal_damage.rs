use common::zones::Zone;

use crate::tests::common_test::cards_with_name;
use crate::tests::common_test::test_state_w_decks;
use anyhow::bail;
use anyhow::Result;
#[test_log::test(tokio::test)]
async fn test_lethal_damage() -> Result<()> {
    let deck = vec!["Staunch Shieldmate"; 60];
    let mut game = test_state_w_decks(deck)?;
    let shieldmates = cards_with_name(&mut game, "Staunch Shieldmate");
    let results = game
        .move_zones(vec![shieldmates[0]], Zone::Library, Zone::Battlefield)
        .await;
    println!("{:?}", results);
    assert!(game.battlefield.len() == 1);
    for (_, player) in game.players.view() {
        assert!(player.graveyard.len() == 0);
    }
    let mut owner = None;
    for &key in &game.battlefield {
        if let Some(card) = game.cards.get_mut(key) {
            card.damaged = 3;
            owner = Some(card.owner);
            break;
        } else {
            bail!("Card wasn't on battlefield");
        }
    }
    game.layers_state_actions().await;
    assert!(game.battlefield.len() == 0);
    let owning_player = game
        .players
        .get(owner.expect("found card"))
        .expect("owner exists");
    assert!(owning_player.graveyard.len() == 1);
    Ok(())
}
