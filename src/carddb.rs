use crate::card_entities::CardEnt;
use crate::card_entities::Supertypes;
use crate::cost::Cost;
use crate::entities::PlayerId;
use crate::mana::ManaCostSymbol;
use anyhow::Result;
use nom::character::complete;
use nom::multi::many0;
use nom::IResult;
use serde_derive::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs;
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
#[allow(dead_code)]
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
            byname.insert(card.name.clone(), card);
        }
        CardDB { scryfall: byname }
    }
    //Precondition: card_name is the name of a valid magic card.
    //Will panic if that is not the case.
    pub fn spawn_card(&self, card_name: &'static str, owner: PlayerId) -> CardEnt {
        let mut card: CardEnt = CardEnt::default();
        card.name = card_name;
        card.owner = owner;
        let scryfall: &ScryfallEntry = self.scryfall.get(card_name).unwrap();
        parse_cost_line(&mut card, scryfall);
        card
    }
}

fn parse_cost_line(card: &mut CardEnt, entry: &ScryfallEntry) {
    if let Some(manatext) = entry.mana_cost.as_ref() {
        let (rest, manas) = parse_mana(&manatext).expect("Failed to parse cost line!");
        if rest.len() > 0 {
            panic!("parser error!");
        }
        card.mana_cost = Some(Cost::Mana(manas));
    }
}

fn parse_manasymbol_contents(input: &str) -> IResult<&str, ManaCostSymbol> {
    let costsymbol = match input {
        "W" => ManaCostSymbol::White,
        "U" => ManaCostSymbol::Blue,
        "B" => ManaCostSymbol::Black,
        "R" => ManaCostSymbol::Red,
        "G" => ManaCostSymbol::Green,
        x => {
            let (rest, cost) = complete::u64(x)?;
            ManaCostSymbol::Generic(cost)
            //A generic cost here
        }
    };
    Ok(("", costsymbol))
}
fn parse_manasymbol(input: &str) -> IResult<&str, ManaCostSymbol> {
    nom::sequence::delimited(
        complete::char('{'),
        parse_manasymbol_contents,
        complete::char('}'),
    )(input)
}

fn parse_mana(input: &str) -> IResult<&str, Vec<ManaCostSymbol>> {
    many0(parse_manasymbol)(input)
}

fn trim_spaces(input: &str) -> IResult<&str, Vec<char>> {
    many0(complete::char(' '))(input)
}
fn parse_supertypes(input: &str) -> IResult<&str, Supertypes> {
    let supertypes = Supertypes::default();
    loop {
        let (input, _) = trim_spaces(input)?;
    }
}
