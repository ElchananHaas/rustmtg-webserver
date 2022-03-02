use crate::ability::Ability;
use crate::card_entities::CardEnt;
use crate::components::Subtype;
use crate::cost::Cost;
use crate::entities::{CardId, PlayerId};
use crate::mana::{mana_cost_string, Color};
use crate::AppendableMap::EntMap;
use anyhow::{bail, Result};
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fs;
use std::hash::Hash;
//It returns mut cardbuilder due to method chaining
pub struct CardDB {
    scryfall: HashMap<String, ScryfallEntry>,
}

impl fmt::Debug for CardDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CardDB").finish()
    }
}
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ScryfallImageUrls {
    small: Option<String>,
    normal: Option<String>,
    large: Option<String>,
}
#[derive(Deserialize, Debug)]
struct ScryfallEntry {
    object: Option<String>,
    name: String,
    image_uris: Option<ScryfallImageUrls>,
    mana_cost: Option<String>,
    type_line: Option<String>,
    lang: Option<String>,
    color_identity: Option<Vec<String>>,
    cmc: Option<f64>,
    power: Option<String>,
    toughness: Option<String>,
}
impl CardDB {
    pub fn new() -> Self {
        let path = "oracle-cards-20211212220409.json";
        let data = fs::read_to_string(path).expect("Couldn't find scryfall oracle database file");
        let desered: Vec<ScryfallEntry> = serde_json::from_str(&data).expect("failed to parse!");
        let mut byname = HashMap::new();
        for card in desered {
            byname.insert(
                card.name.clone(),
                card
            );
        }
        CardDB {
            scryfall: byname,
        }
    }
    //Precondition: card_name is the name of a valid magic card.
    //Will panic if that is not the case.
    pub fn spawn_card(
        &self,
        card_name: &'static str,
        owner: PlayerId,
    ) -> CardEnt {
        let mut card:CardEnt=CardEnt::default();
        card.name=card_name;
        card.owner=owner;
        let scryfall:&ScryfallEntry=self.scryfall.get(card_name).unwrap();
        parse_cost(&mut card,scryfall);
        card
    }
}

fn parse_cost(card: &mut CardEnt,entry:&ScryfallEntry){

}