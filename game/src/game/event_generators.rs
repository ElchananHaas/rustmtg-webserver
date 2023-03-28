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
            if let EventResult::MoveZones(moved) = event {
                for event in moved {
                    if drawn.contains(&&event.oldent)
                    && let Some(newent)=event.newent{
                        inhand.push(newent);
                    }
                }
            }
        }
        inhand
    }

    //discard cards, returns discarded cards
    pub async fn discard(&mut self, player: PlayerId, cards: Vec<CardId>) -> Vec<CardId> {
        let res = self.handle_event(Event::Discard { player, cards }).await;
        let mut discarded = Vec::new();
        for event in &res {
            if let EventResult::MoveZones(moved) = event {
                for event in moved {
                    if event.source==Some(Zone::Hand)
                    && event.dest==Zone::Graveyard
                    && let Some(newent)=event.newent{
                        discarded.push(newent);

                    }
                }
            }
        }
        discarded
    }

    pub async fn move_zones(
        &mut self,
        ents: Vec<CardId>,
        origin: Zone,
        dest: Zone,
    ) -> Vec<EventResult> {
        self.handle_event(Event::MoveZones {
            ents,
            origin: Some(origin),
            dest,
        })
        .await
    }
    pub async fn destroy(&mut self, perms: Vec<CardId>) -> Vec<EventResult> {
        self.handle_event(Event::Destroy { perms }).await
    }
    //Exiles a permanent, records the old and new entities.
    pub async fn exile(&mut self, ents: Vec<CardId>, origin: Zone) -> Vec<EventResult> {
        self.handle_event(Event::MoveZones {
            ents,
            origin: Some(origin),
            dest: Zone::Exile,
        })
        .await
    }
    pub async fn gain_life(&mut self, player: PlayerId, amount: i64) {
        self.handle_event(Event::GainLife { player, amount }).await;
    }
}
