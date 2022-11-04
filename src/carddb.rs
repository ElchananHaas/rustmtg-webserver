use crate::ability::Ability;
use crate::ability::StaticAbility;
use crate::card_entities::CardEnt;
use crate::card_entities::PT;
use crate::card_types::{Subtypes, Supertypes, Types};
use crate::cost::Cost;
use crate::entities::PlayerId;
use crate::mana::ManaCostSymbol;
use crate::spellabil::KeywordAbility;
use anyhow::Result;
use log::debug;
use log::info;
use nom::bytes::complete::take_until;
use nom::character::complete;
use nom::error::ErrorKind;
use nom::multi::many0;
use nom::IResult;
use serde_derive::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::str::FromStr;
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
    oracle_text: Option<String>,
}
impl CardDB {
    pub fn new() -> Self {
        let path = "oracle-cards-20220820210234.json";
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
        self.try_spawn_card(card_name, owner)
            .expect("couldn't spawn card")
    }

    pub fn try_spawn_card(&self, card_name: &'static str, owner: PlayerId) -> Result<CardEnt, ()> {
        info!("spawning {}", card_name);
        let mut card: CardEnt = CardEnt::default();
        card.name = card_name;
        card.printed_name = card_name;
        card.owner = owner;
        let scryfall: &ScryfallEntry = self.scryfall.get(card_name).ok_or(())?;
        parse_cost_line(&mut card, scryfall)?;
        debug!("parsed cost line");
        parse_type_line(&mut card, scryfall)?;
        debug!("parsed type line");
        parse_pt(&mut card, scryfall);
        debug!("parsed P/T");
        parse_body(&mut card, scryfall)?;
        debug!("parsed body");
        card.art_url = (&scryfall.image_uris)
            .as_ref()
            .and_then(|x| x.small.as_ref().or(x.normal.as_ref()).or(x.large.as_ref()))
            .cloned();
        Ok(card)
    }
}

fn parse_body<'a>(card: &mut CardEnt, entry: &'a ScryfallEntry) -> Result<(), ()> {
    if let Some(body) = &entry.oracle_text {
        let tokenized = tokenize(body, Some(&entry.name));
        if let Ok((rest, ())) = parse_body_lines(card, &tokenized) {
            if rest.len() == 0 {
                Ok(())
            } else {
                println!("trailing term {:?}", rest);
                Err(())
            }
        } else {
            Err(())
        }
    } else {
        Ok(())
    }
}
fn parse_keyword_abilities(tokens: &[String]) -> IResult<&[String], Vec<KeywordAbility>, ()> {
    many0(parse_keyword_ability)(tokens)
}
fn parse_keyword_ability(tokens: &[String]) -> IResult<&[String], KeywordAbility, ()> {
    if tokens.len() > 0 {
        if let Ok(abil) = KeywordAbility::from_str(&tokens[0]) {
            let (rest, _) = nom::combinator::opt(tag("\n"))(&tokens[1..])?;
            Ok((rest, abil))
        } else {
            Err(nom::Err::Error(()))
        }
    } else {
        Err(nom::Err::Error(()))
    }
}
fn parse_body_lines<'a>(card: &mut CardEnt, tokens: &'a [String]) -> IResult<&'a [String], (), ()> {
    let (rest, keywords) = parse_keyword_abilities(tokens)?;
    for keyword in keywords {
        card.abilities.push(Ability::Static(StaticAbility {
            keyword: Some(keyword),
        }));
    }
    let (rest, _) = nom::combinator::opt(tag("\n"))(rest)?;
    debug!("rest is {:?}", rest);
    Ok((rest, ()))
}
fn tokenize<'a>(text: &'a str, name: Option<&'a str>) -> Vec<String> {
    let text = if let Some(name) = name {
        text.replace(name, "cardname")
    } else {
        text.to_owned()
    };
    let mut res: Vec<String> = Vec::new();
    for in_line in text.split("\n") {
        let pre_paren: IResult<&str, &str, ()> = take_until("(")(in_line);
        let parse_input;
        if let Ok((_rest, line)) = pre_paren {
            parse_input = line;
        } else {
            parse_input = in_line;
        }
        res.extend(
            parse_input
                .split_whitespace()
                .map(|x| x.to_lowercase())
                .filter(|x| !x.is_empty()),
        );
        res.push(String::from("\n"));
    }
    res
}
fn parse_pt<'a>(card: &mut CardEnt, entry: &'a ScryfallEntry) {
    if let Some(power)=entry.power.as_ref()
    && let Some(toughness)=entry.toughness.as_ref(){
        let res;
        if let Ok(power)=power.parse::<i64>()
        && let Ok(toughness)=toughness.parse::<i64>(){
            res=(power,toughness)
        }else{
            res=(0,0)
        }
        card.pt = Some(PT {
            power: res.0,
            toughness: res.1,
        });
    };
}
fn parse_cost_line<'a>(card: &mut CardEnt, entry: &'a ScryfallEntry) -> Result<(), ()> {
    if let Some(manatext) = entry.mana_cost.as_ref() {
        let (rest, manas) = parse_mana(&manatext).map_err(|_| ())?;
        if rest.len() > 0 {
            panic!("parser error!");
        }
        for mana in manas {
            card.costs.push(Cost::Mana(mana));
        }
        Ok(())
    } else {
        Err(())
    }
}

fn parse_manasymbol_contents(input: &str) -> IResult<&str, Vec<ManaCostSymbol>> {
    if let Ok((rest, symbol)) = complete::one_of::<_, _, (&str, ErrorKind)>("WUBRG")(input) {
        let costsymbol = match symbol {
            'W' => vec![ManaCostSymbol::White],
            'U' => vec![ManaCostSymbol::Blue],
            'B' => vec![ManaCostSymbol::Black],
            'R' => vec![ManaCostSymbol::Red],
            'G' => vec![ManaCostSymbol::Green],
            _ => unreachable!("Already checked symbol"),
        };
        Ok((rest, costsymbol))
    } else {
        complete::u64(input).map(|(rest, x)| (rest, vec![ManaCostSymbol::Generic; x as usize]))
    }
}
fn parse_manasymbol(input: &str) -> IResult<&str, Vec<ManaCostSymbol>> {
    nom::sequence::delimited(
        complete::char('{'),
        parse_manasymbol_contents,
        complete::char('}'),
    )(input)
}

fn parse_mana(input: &str) -> IResult<&str, Vec<ManaCostSymbol>> {
    many0(parse_manasymbol)(input).map(|(rest, x)| (rest, x.into_iter().flatten().collect()))
}

fn parse_type_line<'a>(card: &mut CardEnt, entry: &'a ScryfallEntry) -> Result<(), ()> {
    if let Some(text) = entry.type_line.as_ref() {
        let tokenized = tokenize(&text, None);
        if let Ok((_, (types, subtypes, supertypes))) = parse_type_line_h(&tokenized) {
            card.types = types;
            card.supertypes = supertypes;
            card.subtypes = subtypes;
            Ok(())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}
fn parse_type_line_h<'a>(text: &[String]) -> IResult<&[String], (Types, Subtypes, Supertypes), ()> {
    let (text, supertypes) = Supertypes::parse(text)?;
    let (text, types) = Types::parse(text)?;
    let (text, _) = tag("—")(text)?;
    let (text, subtypes) = Subtypes::parse(text)?;
    Ok((text, (types, subtypes, supertypes)))
}
fn tag(x: &str) -> impl Fn(&[String]) -> IResult<&[String], (), ()> + '_ {
    move |input: &[String]| {
        if input.len() > 0 && input[0] == x.to_lowercase() {
            Ok((&input[1..], ()))
        } else {
            Err(nom::Err::Error(()))
        }
    }
}
