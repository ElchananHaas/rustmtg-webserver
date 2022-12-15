use self::text_token::Tokens;
use crate::ability::Ability;
use crate::ability::StaticAbility;
use crate::card_entities::CardEnt;
use crate::card_entities::PT;
use crate::card_types::Type;
use crate::card_types::{Subtypes, Supertypes, Types};
use crate::cost::Cost;
use crate::entities::PlayerId;
use crate::mana::ManaCostSymbol;
use crate::spellabil::Affected;
use crate::spellabil::Clause;
use crate::spellabil::ClauseConstraint;
use crate::spellabil::ClauseEffect;
use crate::spellabil::KeywordAbility;
use crate::token_builder::parse_token_attributes;
use log::debug;
use log::info;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::one_of;
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
use spawn_error::SpawnError;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::str::FromStr;
pub mod spawn_error;
pub mod text_token;
use crate::tokens;
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
    tokenized_type_line: Option<Vec<String>>, //Will be tokeized upon construction
    lang: Option<String>,
    color_identity: Option<Vec<String>>,
    cmc: Option<f64>,
    power: Option<String>,
    toughness: Option<String>,
    oracle_text: Option<String>,
    tokenized_oracle_text: Option<Vec<String>>, //Will be tokeized upon construction
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
        let path = "oracle-cards-20220820210234.json";
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
pub fn parse_number<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, i64> {
    if tokens.len() == 0 {
        return Err(nom_error(tokens, "Empty tokens when parsing integer"));
    }
    let first_token = &tokens[0];
    if let Ok(num) = i64::from_str(&first_token) {
        Ok((&tokens[1..], num))
    } else {
        Err(nom_error(tokens, "Failed to parse integer"))
    }
}
fn parse_gain_life<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, _) = tag(tokens!("you", "gain"))(tokens)?;
    let (tokens, value) = parse_number(tokens)?;
    let (tokens, _) = tag(tokens!("life"))(tokens)?;
    Ok((
        tokens,
        Clause {
            effect: ClauseEffect::GainLife(value),
            affected: Affected::Controller,
            constraints: Vec::new(),
        },
    ))
}
fn parse_draw_a_card<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, _) = tag(tokens!("draw", "a", "card"))(tokens)?;
    Ok((
        tokens,
        Clause {
            effect: ClauseEffect::DrawCard,
            affected: Affected::Controller,
            constraints: Vec::new(),
        },
    ))
}
fn parse_body_line<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, clause) = context(
        "parsing body line",
        alt((
            parse_gain_life,
            parse_draw_a_card,
            parse_target_line,
            parse_create_token,
        )),
    )(tokens)?;
    let (tokens, _) = opt(tag(tokens!(".")))(tokens)?;
    let (tokens, _) = opt(tag(tokens!("\n")))(tokens)?;
    Ok((tokens, clause))
}

fn parse_create_token<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, _) = alt((tag(tokens!["create"]), tag(tokens!["creates"])))(tokens)?;
    let (tokens, _) = tag(tokens!["a"])(tokens)?;
    let (tokens, attr1) = parse_token_attributes(tokens)?;
    let (tokens, _) = tag(tokens!["token"])(tokens)?;
    let (tokens, _) = opt(tag(tokens!["with"]))(tokens)?;
    let (tokens, attr2) = parse_token_attributes(tokens)?;
    let attrs = attr1.into_iter().chain(attr2.into_iter()).collect();
    Ok((
        tokens,
        Clause {
            effect: ClauseEffect::CreateToken(attrs),
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
fn parse_target_line<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, clause) = context(
        "parsing target line",
        alt((parse_destroy_clause, parse_exile_clause)),
    )(tokens)?;
    let (tokens, addendum) = opt(parse_its_controller_clause)(tokens)?;
    if let Some(addendum) = addendum {
        Ok((
            tokens,
            Clause {
                effect: ClauseEffect::Compound(vec![clause.clone(), addendum]),
                affected: clause.affected,
                constraints: clause.constraints,
            },
        ))
    } else {
        Ok((tokens, clause))
    }
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

fn parse_destroy_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, _) = context("parsing destroy target", tag(tokens!["destroy", "target"]))(tokens)?;
    let (tokens, constraints) = many1(parse_constraint)(tokens)?;
    Ok((
        tokens,
        Clause {
            effect: ClauseEffect::Destroy,
            constraints,
            affected: Affected::Target(None),
        },
    ))
}
fn parse_exile_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, _) = context("parsing exile clause", tag(tokens!["exile", "target"]))(tokens)?;
    let (tokens, constraints) = many1(parse_constraint)(tokens)?;
    Ok((
        tokens,
        Clause {
            effect: ClauseEffect::ExileBattlefield,
            constraints,
            affected: Affected::Target(None),
        },
    ))
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
fn parse_token<'a>(mut text: &'a str) -> IResult<&str, String, ()> {
    let special_chars = " .:,\"\n()/";
    (text, _) = many0(nom::character::complete::char(' '))(text)?;
    if let Ok((rest, char)) = one_of::<_, _, ()>(special_chars)(text) {
        if char == ' ' {
            text = rest;
        } else {
            return Ok((rest, char.to_string()));
        }
    };
    let (rest, word) = is_not::<_, _, ()>(special_chars)(text)?;
    if word.len() > 0 {
        return Ok((rest, word.to_lowercase()));
    }
    return Err(nom::Err::Error(()));
}
fn tokenize<'a>(text: &'a str, name: Option<&'a str>) -> Vec<String> {
    let text = if let Some(name) = name {
        text.replace(name, "cardname")
    } else {
        text.to_owned()
    };
    let (remainder, res) = many0(parse_token)(&text).expect("Tokenizing failed");
    assert!(remainder.len() == 0);
    return res;
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
fn parse_cost_line<'a>(
    card: &mut CardEnt,
    entry: &'a ScryfallEntry,
) -> Result<(), nom::Err<VerboseError<&'a str>>> {
    if let Some(manatext) = entry.mana_cost.as_ref() {
        let (rest, manas) = parse_mana(&manatext)?;
        if rest.len() > 0 {
            panic!("parser error!");
        }
        for mana in manas {
            card.costs.push(Cost::Mana(mana));
        }
        Ok(())
    } else {
        Err(nom::Err::Error(VerboseError {
            errors: vec![(
                "",
                nom::error::VerboseErrorKind::Context("card had no cost line"),
            )],
        }))
    }
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

fn parse_mana(input: &str) -> Res<&str, Vec<ManaCostSymbol>> {
    many0(parse_manasymbol)(input).map(|(rest, x)| (rest, x.into_iter().flatten().collect()))
}

fn parse_type_line<'a>(card: &mut CardEnt, entry: &'a ScryfallEntry) -> Res<&'a Tokens, ()> {
    if let Some(tokenized) = entry.tokenized_type_line.as_ref() {
        let tokens = Tokens::from_array(&tokenized);
        let (rest, (types, subtypes, supertypes)) = parse_type_line_h(&tokens)?;
        if rest.len() == 0 {
            card.types = types;
            card.supertypes = supertypes;
            card.subtypes = subtypes;
            Ok((rest, ()))
        } else {
            Err(nom_error(rest, "failed to parse complete type line"))
        }
    } else {
        Err(nom_error(
            &Tokens::empty(),
            "scryfall entry had no type line",
        ))
    }
}
fn parse_type_line_h<'a>(text: &'a Tokens) -> Res<&'a Tokens, (Types, Subtypes, Supertypes)> {
    let (text, supertypes) = Supertypes::parse(text)?;
    let (text, types) = Types::parse(text)?;
    let (text, _) = opt(tag(Tokens::from_array(&["—".to_owned()])))(text)?;
    let (text, subtypes) = Subtypes::parse(text)?;
    let (text, _) = opt(tag(Tokens::from_array(&["\n".to_owned()])))(text)?;
    Ok((text, (types, subtypes, supertypes)))
}

mod tests {
    use std::num::NonZeroU64;

    use once_cell::sync::OnceCell;

    use super::*;
    static CARDDB: OnceCell<CardDB> = OnceCell::new();

    #[test_log::test]
    fn card_tests() {
        test_card(db(), "Staunch Shieldmate");
        test_card(db(), "Plains");
        test_card(db(), "Revitalize");
    }
    fn db() -> &'static CardDB {
        CARDDB.get_or_init(|| CardDB::new())
    }
    #[test_log::test]
    fn revitalize_test() {
        test_card(db(), "Revitalize");
    }
    #[test_log::test]
    fn defiant_strike_test() {
        test_card(db(), "Defiant Strike");
    }
    #[test_log::test]
    fn swift_response_test() {
        test_card(db(), "Swift Response");
    }
    #[test_log::test]
    fn angelic_ascension_test() {
        test_card(db(), "Angelic Ascension");
    }
    #[allow(dead_code)]
    fn test_card(db: &CardDB, card_name: &'static str) -> CardEnt {
        let spawned = db.try_spawn_card(card_name, PlayerId::from(NonZeroU64::new(1).unwrap()));
        if spawned.is_err() {
            println!("card {} failed to spawn", card_name);
        }
        if let Err(err)=&spawned
        && let SpawnError::Nom(err)=err
        && let nom::Err::Error(err)=err{
            println!("spawn error[");
            for error in &err.errors{
                println!("{:?}",error);
            } 
        }
        let spawned = spawned.unwrap();
        //println!("{:?}", spawned);
        spawned
    }
}
