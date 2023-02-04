use crate::game::*;

impl Game {
    //Taps an entity, returns if it was sucsessfully tapped
    pub async fn tap(&mut self, ent: CardId) -> bool {
        self.handle_event(Event::Tap { ent })
            .await
            .contains(&EventResult::Tap(ent))
    }
    //Taps an entity, returns if it was sucsessfully tapped
    pub async fn untap(&mut self, ent: CardId) -> bool {
        self.handle_event(Event::Untap { ent })
            .await
            .contains(&EventResult::Untap(ent))
    }
    //draws a card, returns the entities drawn
    pub async fn draw(&mut self, player: PlayerId) -> Vec<CardId> {
        let res = self.handle_event(Event::Draw { player }).await;
        let mut drawn = Vec::new();
        for event in &res {
            if let EventResult::Draw(cardid) = event {
                if let Some(card) = self.cards.get(*cardid) {
                    if card.owner == player {
                        drawn.push(cardid);
                    }
                }
            }
        }
        let mut inhand = Vec::new();
        for event in &res {
            if let EventResult::MoveZones {
                oldent,
                newent,
                source: _,
                dest: _,
            } = event
            {
                if drawn.contains(&oldent)
                && let Some(newent)=newent{
                    inhand.push(*newent);
                }
            }
        }
        inhand
    }

    //discard cards, returns discarded cards
    pub async fn discard(
        &mut self,
        player: PlayerId,
        card: CardId,
        cause: DiscardCause,
    ) -> Vec<CardId> {
        let res = self
            .handle_event(Event::Discard {
                player,
                card,
                cause,
            })
            .await;
        let mut discarded = Vec::new();
        for event in res {
            if let EventResult::MoveZones {
                oldent: _,
                newent: Some(newent),
                source: Some(Zone::Hand),
                dest: Zone::Graveyard,
            } = event
            {
                discarded.push(newent);
            }
        }
        discarded
    }

    pub async fn move_zones(&mut self, ent: CardId, origin: Zone, dest: Zone) -> Vec<EventResult> {
        self.handle_event(Event::MoveZones {
            ent,
            origin: Some(origin),
            dest,
        })
        .await
    }
    pub async fn destroy(&mut self, id: CardId) -> Vec<EventResult> {
        self.handle_event(Event::Destroy { card: id }).await
    }
    //Exiles a permanent, records the old and new entities.
    pub async fn exile(&mut self, id: CardId, origin: Zone) -> Vec<EventResult> {
        self.handle_event(Event::MoveZones {
            ent: id,
            origin: Some(origin),
            dest: Zone::Exile,
        })
        .await
    }
    pub async fn gain_life(&mut self, player: PlayerId, amount: i64) {
        self.handle_event(Event::GainLife { player, amount }).await;
    }
}
