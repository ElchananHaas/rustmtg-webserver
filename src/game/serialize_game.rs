use crate::{
    client_message::{ClientMessage, GameState},
    game::*,
};
use rand::prelude::*;
use std::num::NonZeroU64;
impl Game {
    pub async fn send_state(&mut self) {
        let mut state_futures = Vec::new();
        for player in self.turn_order.clone() {
            state_futures.push(self.send_state_player(player));
        }
        let _results = future::join_all(state_futures).await;
    }
    async fn send_state_player(&self, player: PlayerId) -> Result<()> {
        let mut card_views = HashMap::new();
        let mut hidden_ids = Vec::new();
        let mut dummy_cards = Vec::new();
        for (card_id, card_ref) in self.cards.view() {
            if card_ref.known_to.contains(&player) {
                card_views.insert(card_id, card_ref);
            } else {
                hidden_ids.push(card_id);
                let mut dummy_card = CardEnt::default();
                dummy_card.owner = card_ref.owner;
                dummy_cards.push(dummy_card);
            }
        }
        let mut hidden_map: HashMap<CardId, CardId> = HashMap::new();
        {
            let mut rng = thread_rng();
            hidden_ids.shuffle(&mut rng);
            let mut next_id = self.cards.peek_count();
            for i in 0..hidden_ids.len() {
                next_id = NonZeroU64::new((u64::from(next_id)) + 1).unwrap();
                let card_id = next_id.into();
                card_views.insert(card_id, &dummy_cards[i]);
                hidden_map.insert(hidden_ids[i], card_id);
            }
        }
        let mut player_views = HashMap::new();
        for (player_id, player_ref) in self.players.view() {
            let view = player_ref.view(&self.cards, player, &hidden_map);
            player_views.insert(player_id, view);
        }
        if let Some(pl) = self.players.get(player) {
            let message: ClientMessage = ClientMessage::GameState(GameState {
                player,
                cards: card_views,
                players: player_views,
                game: self,
            });
            pl.send_data(message).await?;
        }
        Ok(())
    }
}
//This struct maps the hidden CardId's to the ones exposed to the player.
//If the player knows the information the public ID is used, otherwise
//it generates one.
