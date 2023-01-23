use crate::parse_clauseeffect::parse_action_first_effect;
use crate::parse_clauseeffect::parse_action_second_effect;
use crate::parse_non_body::parse_cost_line;
use crate::parse_non_body::parse_pt;
use crate::parse_non_body::parse_type_line;
use crate::spawn_error::SpawnError;
use crate::tokenize::tokenize;
use cardtypes::Type;
use common::ability::Ability;
use common::card_entities::CardEnt;
use common::entities::PlayerId;
use common::mana::ManaCostSymbol;
use common::spellabil::Affected;
use common::spellabil::Clause;
use common::spellabil::ClauseConstraint;
use common::spellabil::ClauseEffect;
use common::spellabil::KeywordAbility;
use log::debug;
use log::info;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::combinator::opt;
use nom::error::context;
use nom::error::ErrorKind;
use nom::error::VerboseError;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::IResult;
use serde_derive::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::str::FromStr;
use texttoken::{tokens, Tokens};
pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

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
pub struct ScryfallImageUrls {
    pub small: Option<String>,
    pub normal: Option<String>,
    pub large: Option<String>,
}
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ScryfallEntry {
    pub object: Option<String>,
    pub name: String,
    pub image_uris: Option<ScryfallImageUrls>,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub tokenized_type_line: Option<Vec<String>>, //Will be tokeized upon construction
    pub lang: Option<String>,
    pub color_identity: Option<Vec<String>>,
    pub cmc: Option<f64>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub oracle_text: Option<String>,
    pub tokenized_oracle_text: Option<Vec<String>>, //Will be tokeized upon construction
}
pub fn nom_error<'a>(
    tokens: &'a Tokens,
    message: &'static str,
) -> nom::Err<VerboseError<&'a Tokens>> {
    nom::Err::Error(VerboseError {
        errors: vec![(tokens, nom::error::VerboseErrorKind::Context(message))],
    })
}

impl CardDB {
    pub fn new() -> Self {
        let path = "../oracle-cards-20230120100202.json";
        let data = fs::read_to_string(path).expect("Couldn't find scryfall oracle database file");
        let desered: Vec<ScryfallEntry> = serde_json::from_str(&data).expect("failed to parse!");
        let mut byname = HashMap::new();
        for mut card in desered {
            card.tokenized_type_line = card.type_line.as_ref().map(|line| tokenize(line, None));
            card.tokenized_oracle_text = card
                .oracle_text
                .as_ref()
                .map(|line| tokenize(line, Some(&card.name)));
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

    pub fn try_spawn_card(
        &self,
        card_name: &'static str,
        owner: PlayerId,
    ) -> Result<CardEnt, SpawnError> {
        info!("spawning {}", card_name);
        let mut card: CardEnt = CardEnt::default();
        card.name = card_name;
        card.printed_name = card_name;
        card.owner = owner;
        let scryfall: &ScryfallEntry = self
            .scryfall
            .get(card_name)
            .ok_or(SpawnError::CardNotFoundError(card_name))?;
        parse_cost_line(&mut card, scryfall).unwrap();
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
        card.printed = Some(Box::new(card.clone()));
        Ok(card)
    }
}

fn parse_body<'a>(
    card: &mut CardEnt,
    entry: &'a ScryfallEntry,
) -> Result<(), nom::Err<VerboseError<&'a Tokens>>> {
    if let Some(tokenized) = &entry.tokenized_oracle_text {
        let tokens = Tokens::from_array(&tokenized);
        let (rest, ()) = parse_body_lines(card, tokens)?;
        assert!(rest.len() == 0); //parse body lines loops until rest is empty
        Ok(())
    } else {
        Ok(())
    }
}
fn parse_keyword_abilities<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<KeywordAbility>> {
    many0(parse_keyword_ability)(tokens)
}
fn parse_keyword_ability<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, KeywordAbility> {
    if tokens.len() > 0 {
        if let Ok(abil) = KeywordAbility::from_str(&tokens[0]) {
            let (rest, _) =
                nom::combinator::opt(tag(Tokens::from_array(&["\n".to_string()])))(&tokens[1..])?;
            Ok((rest, abil))
        } else {
            Err(nom_error(tokens, "failed to parse keyword ability"))
        }
    } else {
        Err(nom_error(tokens, "Empty string passed to keyword ability"))
    }
}
fn prune_comment<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ()> {
    let (rest, _) = opt(delimited(
        tag(tokens!["("]),
        is_not(tokens!(")")),
        tag(tokens!(")")),
    ))(tokens)?;
    Ok((rest, ()))
}

fn parse_body_line<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, clause) = context(
        "parsing body line",
        alt((
            parse_you_clause,
            parse_action_target_line,
            parse_target_action_line,
        )),
    )(tokens)?;
    let (tokens, _) = opt(tag(tokens!(".")))(tokens)?;
    let (tokens, _) = opt(tag(tokens!("\n")))(tokens)?;
    Ok((tokens, clause))
}
fn parse_you_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, _) = opt(tag(tokens!["you"]))(tokens)?;
    //Sometimes MTG
    //Implicitly has a clause mean you if it is left out.
    //For example, "draw a card" vs. "you draw a card"
    let (tokens, effect) = parse_action_second_effect(tokens)?;
    Ok((
        tokens,
        Clause {
            effect,
            affected: Affected::Controller,
            constraints: Vec::new(),
        },
    ))
}
fn parse_its_controller_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, _) = tag(tokens!["."])(tokens)?;
    let (tokens, _) = opt(tag(tokens!("\n")))(tokens)?;
    let (tokens, _) = context(
        "parsing controller addon",
        tag(tokens!["its", "controller"]),
    )(tokens)?;
    let (tokens, clause) = parse_body_line(tokens)?;
    Ok((
        tokens,
        Clause {
            affected: Affected::Target(None),
            effect: ClauseEffect::SetTargetController(Box::new(clause)),
            constraints: vec![],
        },
    ))
}
fn parse_target_action_line<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, _) = tag(tokens!["target"])(tokens)?;
    let (tokens, constraints) = many1(parse_constraint)(tokens)?;
    let (tokens, effect) = context("parsing target line", parse_action_second_effect)(tokens)?;
    let clause = Clause {
        effect,
        affected: Affected::Target(None),
        constraints,
    };
    Ok((tokens, clause))
}

fn parse_action_target_line<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, effect) = context("parsing target line", parse_action_first_effect)(tokens)?;
    let (tokens, _) = tag(tokens!["target"])(tokens)?;
    let (tokens, constraints) = many1(parse_constraint)(tokens)?;
    let (tokens, addendum) = opt(parse_its_controller_clause)(tokens)?;
    let mut clause = Clause {
        effect,
        affected: Affected::Target(None),
        constraints,
    };
    if let Some(addendum) = addendum {
        clause.effect = ClauseEffect::Compound(vec![clause.clone(), addendum]);
    }
    Ok((tokens, clause))
}

fn parse_tapped_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseConstraint> {
    let (tokens, _) = tag(tokens!["tapped"])(tokens)?;
    Ok((tokens, ClauseConstraint::IsTapped))
}
fn parse_type<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Type> {
    let first = &*tokens[0];
    let rest = &tokens[1..];
    if let Ok(t) = Type::from_str(first) {
        return Ok((rest, t));
    }
    Err(nom_error(tokens, "Not a type"))
}
fn parse_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseConstraint> {
    let type_constraint = nom::combinator::map(parse_type, |t| ClauseConstraint::CardType(t));
    let (tokens, constraint) = alt((parse_tapped_constraint, type_constraint))(tokens)?;
    let (tokens, or_part) = opt(parse_or_constraint)(tokens)?;
    if let Some(or_part) = or_part {
        Ok((tokens, ClauseConstraint::Or(vec![constraint, or_part])))
    } else {
        Ok((tokens, constraint))
    }
}
fn parse_or_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseConstraint> {
    let (tokens, _) = tag(tokens!["or"])(tokens)?;
    let (tokens, constraint) = parse_constraint(tokens)?;
    Ok((tokens, constraint))
}

fn parse_body_lines<'a>(card: &mut CardEnt, tokens: &'a Tokens) -> Res<&'a Tokens, ()> {
    let (mut rest, keywords) = context("parse keywords", parse_keyword_abilities)(tokens)?;
    for keyword in keywords {
        card.abilities.push(Ability::from_keyword(keyword));
    }
    let mut clauses = Vec::new();
    while rest.len() > 0 {
        let clause;
        (rest, _) = context("pruning comments", prune_comment)(rest)?;
        if rest.len() == 0 {
            break;
        }
        (rest, clause) = parse_body_line(rest)?;
        clauses.push(clause);
    }
    card.effect = clauses;
    let (rest, _) = nom::combinator::opt(tag(tokens!["\n"]))(rest)?;
    Ok((rest, ()))
}

fn parse_manasymbol_contents(input: &str) -> Res<&str, Vec<ManaCostSymbol>> {
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
fn parse_manasymbol(input: &str) -> Res<&str, Vec<ManaCostSymbol>> {
    nom::sequence::delimited(
        complete::char('{'),
        parse_manasymbol_contents,
        complete::char('}'),
    )(input)
}

pub fn parse_mana(input: &str) -> Res<&str, Vec<ManaCostSymbol>> {
    many0(parse_manasymbol)(input).map(|(rest, x)| (rest, x.into_iter().flatten().collect()))
}
