use crate::ability::Ability;
use crate::ability::KeywordAbility;
use crate::carddb::{CardDB,CardIdentity};
use crate::event::{Event, EventCause, EventResult, TagEvent};
use crate::types::Types;
use anyhow::{bail, Result};
use hecs::{Entity, World};
use std::collections::HashSet;
pub struct GameBuilder {
    ents: World,
    turn_order: Vec<Entity>,
    active_player: Option<Entity>,
}
//Implement debug trait!
//Implement clone trait???
pub struct Game<'a> {
    pub ents: World,
    pub battlefield: HashSet<Entity>,
    pub exile: HashSet<Entity>,
    pub command: HashSet<Entity>,
    pub stack: Vec<Entity>,
    pub turn_order: Vec<Entity>,
    pub active_player: Entity,
    db: &'a CardDB
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
        let mut cards = Vec::new();
        let player = Player {
            name: name.to_owned(),
            hand: HashSet::new(),
            life: 20,
            mana_pool: HashSet::new(),
            graveyard: Vec::new(),
            lost: false,
            won: false,
            deck: Vec::new(),
        };
        let player: Entity = self.ents.spawn((player,));
        for cardname in card_names {
            let card: Entity = db.spawn_card(&mut self.ents, &cardname)?;
            self.ents.insert(card, (Owner(player),))?;
            cards.push(card);
        }
        //Now that the deck has been constructed, set the players deck
        self.ents.get_mut::<Player>(player).unwrap().deck = cards;
        self.turn_order.push(player);
        if self.active_player.is_none() {
            self.active_player = Some(player);
        }
        Ok(player)
    }
    pub fn build<'a>(self,db: &'a CardDB) -> Result<Game> {
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
            db:db
        })
    }
}
impl<'a> Game<'a> {
    fn handle_event(&mut self, event: Event, cause: EventCause) -> Vec<EventResult> {
        let mut results: Vec<EventResult> = Vec::new();
        let mut events: Vec<TagEvent> = Vec::new();
        events.push(TagEvent {
            event: event,
            replacements: Vec::new(),
            cause: cause,
        });
        loop {
            let event: TagEvent = match events.pop() {
                Some(x) => x,
                None => {
                    return results;
                }
            };
            //Handle prevention, replacement, triggered abilties here
            //By the time the loop reaches here, the game is ready to
            //Execute the event. No more prevention/replacement effects
            match event.event {
                Event::Tap { ent } => {
                    if self.battlefield.contains(&ent) && self.ents.get::<Tapped>(ent).is_err() {
                        if self.ents.insert_one(ent, Tapped()).is_ok() {
                            results.push(EventResult::Tap(ent));
                        }
                    }
                }
                Event::Draw { player, controller } => {
                    if let Ok(pl) = self.ents.get::<Player>(player) {
                        match pl.deck.last() {
                            Some(card) => {
                                Game::add_event(
                                    &mut events,
                                    Event::MoveZones {
                                        ent: *card,
                                        origin: Zone::Library,
                                        dest: Zone::Hand,
                                    },
                                    event.cause,
                                );
                                results.push(EventResult::Draw(*card));
                            }
                            None => Game::add_event(
                                &mut events,
                                Event::Lose { player: player },
                                EventCause::Trigger(event.event.clone()),
                            ),
                        }
                    }
                }
                Event::Cast { player: _, spell: _ } => {
                    //The spell has already had costs/modes chosen.
                    //this is just handling triggered abilities
                    //So there is nothing to do here.
                    //Spells are handled differently from other actions
                    //Because of the rules complexity
                }
                Event::Activate { controller: _, ability: _ } => {
                    //Similar to spell casting
                }
                Event::Lose { player } => {
                    if let Ok(mut pl) = self.ents.get_mut::<Player>(player) {
                        (*pl).lost = true;
                    }
                }
                Event::MoveZones { ent, origin, dest } => {
                    if origin == dest {
                        continue;
                    };
                    let mut truename=None;
                    if let Ok(owner) = self.ents.get::<Owner>(ent) {
                        if let Ok(mut player) = self.ents.get_mut::<Player>(owner.0) {
                            let removed = match origin {
                                Zone::Exile => self.exile.remove(&ent),
                                Zone::Command => self.command.remove(&ent),
                                Zone::Battlefield => self.battlefield.remove(&ent),
                                Zone::Hand => player.hand.remove(&ent),
                                Zone::Library => match player.deck.iter().position(|x| *x == ent) {
                                    Some(i) => {
                                        player.deck.remove(i);
                                        true
                                    }
                                    None => false,
                                },
                                Zone::Graveyard => {
                                    match player.graveyard.iter().position(|x| *x == ent) {
                                        Some(i) => {
                                            player.graveyard.remove(i);
                                            true
                                        }
                                        None => false,
                                    }
                                }
                                Zone::Stack => match self.stack.iter().position(|x| *x == ent) {
                                    Some(i) => {
                                        self.stack.remove(i);
                                        true
                                    }
                                    None => false,
                                },
                            };
                            if removed{
                                if let Ok(iden)=self.ents.get::<CardIdentity>(ent){
                                    if !iden.token{
                                        truename=Some(iden.name.clone());
                                    }
                                }
                            }
                        } else {
                            panic!("Owners must be players");
                        }
                    } else {
                        panic!("All entities need an owner");
                    }
                    if let Some(name)=truename{
                        let newent=self.db.spawn_card(&mut self.ents,&name).unwrap();
                        let owner = self.ents.get::<Owner>(ent).unwrap();
                        let mut player = self.ents.get_mut::<Player>(owner.0).unwrap();
                        match dest{
                            Zone::Exile => {self.exile.insert(newent);},
                            Zone::Command => {self.command.insert(newent);},
                            Zone::Battlefield => {self.battlefield.insert(newent);},
                            Zone::Hand => {player.hand.insert(newent);},
                            //Handle inserting a distance from the top. Perhaps swap them afterwards?
                            Zone::Library => player.deck.push(newent),
                            Zone::Graveyard => player.graveyard.push(newent),
                            Zone::Stack => self.stack.push(newent),
                        }
                    }
                }
            }
        }
    }

    fn add_event(events: &mut Vec<TagEvent>, event: Event, cause: EventCause) {
        events.push(TagEvent {
            event: event,
            replacements: Vec::new(),
            cause: cause,
        });
    }
    pub fn tap(&mut self, ent: Entity, cause: EventCause) -> bool {
        self.handle_event(Event::Tap { ent: ent }, cause)
            .contains(&EventResult::Tap(ent))
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
pub struct Player {
    pub name: String,
    pub life: i32,
    pub deck: Vec<Entity>,
    pub hand: HashSet<Entity>,
    pub mana_pool: HashSet<Entity>,
    pub graveyard: Vec<Entity>,
    pub lost: bool,
    pub won: bool,
}
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
