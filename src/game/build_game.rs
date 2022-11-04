use crate::game::*;
pub struct GameBuilder {
    players: Players,
    cards: Cards,
    turn_order: VecDeque<PlayerId>,
}

impl GameBuilder {
    pub fn new() -> Self {
        let mut cards = Cards::new();
        cards.skip_count(MIN_CARDID); //make sure they don't overlap with players in JavaScript
        GameBuilder {
            players: Players::new(),
            cards,
            turn_order: VecDeque::new(),
        }
    }
    //If this function fails the game is corrupted
    pub fn add_player(
        &mut self,
        name: &str,
        db: &CardDB,
        card_names: &Vec<&'static str>,
        player_con: PlayerCon,
    ) -> Result<PlayerId> {
        let mut cards = Vec::new();
        let player = Player {
            name: name.to_owned(),
            hand: HashSet::new(),
            life: 20,
            mana_pool: HashSet::new(),
            graveyard: Vec::new(),
            library: Vec::new(),
            max_handsize: 7,
            player_con: player_con,
        };
        let (player_id, player) = self.players.insert(player);
        for cardname in card_names {
            let card: CardEnt = db.spawn_card(cardname, player_id);
            let (card_id, _card) = self.cards.insert(card);
            cards.push(card_id);
        }
        //Now that the deck has been constructed, set the players deck
        player.library = cards;
        self.turn_order.push_back(player_id);
        Ok(player_id)
    }
    pub fn build(self, db: &'static CardDB) -> Result<Game> {
        if self.turn_order.len() < 2 {
            bail!("Game needs at least two players in initialization")
        };
        let start = self.turn_order[0];
        Ok(Game {
            players: self.players,
            cards: self.cards,
            mana: EntMap::new(),
            battlefield: HashSet::new(),
            exile: HashSet::new(),
            command: HashSet::new(),
            stack: Vec::new(),
            turn_order: self.turn_order,
            active_player: start,
            db,
            land_play_limit: 1,
            lands_played_this_turn: 0,
            extra_turns: VecDeque::new(),
            phases: VecDeque::new(),
            subphases: VecDeque::new(),
            phase: None,
            subphase: None,
            priority: start,
            outcome: GameOutcome::Ongoing,
            backup: None,
            rng: rand::rngs::StdRng::from_entropy(),
        })
    }
}
