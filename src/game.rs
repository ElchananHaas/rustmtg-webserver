use crate::ability::Ability;
use crate::ability::KeywordAbility;
use crate::carddb::CardDB;
use crate::event::{Event, EventResult, TagEvent};
use crate::types::Types;
use anyhow::{bail, Result};
use hecs::{Entity, World};
use std::collections::HashSet;
use std::collections::VecDeque;
pub struct GameBuilder {
    ents: World,
    turn_order: Vec<Entity>,
    active_player: Option<Entity>,
}
//Implement debug trait!
//Implement clone trait???
pub struct Game {
    pub ents: World,
    pub battlefield: HashSet<Entity>,
    pub exile: HashSet<Entity>,
    pub command: HashSet<Entity>,
    pub stack: Vec<Entity>,
    pub turn_order: Vec<Entity>,
    pub active_player: Entity,
    event_stack: Vec<TagEvent>,
}

impl GameBuilder {
    pub fn new() -> Self {
        GameBuilder {
            ents: World::new(),
            turn_order: Vec::new(),
            active_player: None,
        }
    }
    //If this function fails the game is corrupted
    //Potentialy fail game creation if player can't be added?
    pub fn add_player(
        &mut self,
        name: &str,
        db: &CardDB,
        card_names: &Vec<String>,
    ) -> Result<Entity> {
        let mut cards = VecDeque::new();
        let name = PlayerName(name.to_owned());
        let hand = Hand(HashSet::new());
        let life = Life(20);
        let pool = ManaPool(HashSet::new());
        let player: Entity = self.ents.spawn((name, hand, life, pool));
        for cardname in card_names {
            let card: Entity = db.spawn_card(&mut self.ents, &cardname)?;
            self.ents.insert_one(card, (Owner(player),Zone::Library))?;
            cards.push_back(card);
        }
        let deck = Deck(cards);
        self.ents.insert_one(player, deck)?;
        self.turn_order.push(player);
        if self.active_player.is_none() {
            self.active_player = Some(player);
        }
        Ok(player)
    }
    pub fn build(self) -> Result<Game> {
        let active_player = match self.active_player {
            Some(player) => player,
            None => {
                bail!("Active player must be set in game initilization");
            }
        };
        if self.turn_order.len() < 2 {
            bail!("Game needs at least two players in initilization")
        };
        Ok(Game {
            ents: self.ents,
            battlefield: HashSet::new(),
            exile: HashSet::new(),
            command: HashSet::new(),
            stack: Vec::new(),
            turn_order: self.turn_order,
            active_player: active_player,
        })
    }
}
impl Game {
    fn resolve_events(&mut self) -> Vec<EventResult> {
        let results: Vec<EventResult> = Vec::new();
        loop {
            let mut event: TagEvent = match game.stack.pop() {
                Some(x) => x,
                None => {
                    return results;
                }
            };
            //Handle prevention, replacement, triggered abilties here
            //By the time the loop reaches here, the game is ready to
            //Execute the event. No more prevention/replacement effects
            match event.event {
                Event::Draw { player, _ignore } => {
                    if let Ok(deck) = self.ents.get::<Deck>(player) {
                        match deck.front() {
                            Some(card) =>{
                                self.move_ent(card,Zone::Library,Zone::Hand);
                            },
                            None => self.add_event(Lose{player:player}),
                        }
                    }
                }
            }
        }
    }
    fn add_event(&mut self, event: Event) {
        game.event_stack.push(TagEvent {
            event: event,
            replacements: Vec::new(),
        });
    }
    //Wraps and adds an event to the event stack
    pub fn handle_event(&mut self, event: Event) {
        self.add_event(event);
        game.resolve_events();
    }
    //Can this creature tap to be declared an attacker or to activate an ability?
    //Doesn't include prevention effects, just if it can tap w/o them
    pub fn can_tap(&self, ent: Entity) -> bool {
        if self.ents.get::<Tapped>(ent).is_ok() {
            return false;
        }
        if let Ok(types) = self.ents.get::<Types>(ent) {
            if types.creature {
                if self.ents.get::<SummoningSickness>(ent).is_ok() {
                    if let Ok(abilities) = self.ents.get::<Vec<Ability>>(ent) {
                        for ability in &(*abilities) {
                            if ability.keyword() == Some(KeywordAbility::Haste) {
                                return true;
                            }
                        }
                        return false;
                    } else {
                        false
                    }
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            false
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Owner(pub Entity);
#[derive(Clone, Debug)]
pub struct PlayerName(pub String);
#[derive(Clone, Copy, Debug)]
pub struct Life(pub i32);
#[derive(Clone, Debug)]
pub struct Deck(pub VecDeque<Entity>);
#[derive(Clone, Debug)]
pub struct Hand(pub HashSet<Entity>);
#[derive(Clone, Debug)]
pub struct ManaPool(pub HashSet<Entity>);

//Entered or changed control, use the game function
//to check if it has summoning sickness
pub struct SummoningSickness();
pub struct Tapped();
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Zone {
    Hand,
    Library,
    Exile,
    Battlefield,
    Graveyard,
    Command,
    Stack,
}
