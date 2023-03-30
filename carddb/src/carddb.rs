use crate::parse_clause::parse_clause;
use crate::parse_constraint::parse_constraint;
use crate::parse_non_body::parse_cost_line;
use crate::parse_non_body::parse_pt;
use crate::parse_non_body::parse_type_line;
use crate::spawn_error::SpawnError;
use crate::tokenize::tokenize;
use crate::util::parse_number;
use common::ability::Ability;
use common::ability::AbilityTrigger;
use common::ability::AbilityTriggerType;
use common::ability::ActivatedAbility;
use common::ability::Replacement;
use common::ability::ReplacementAbility;
use common::ability::StaticAbility;
use common::ability::StaticAbilityEffect;
use common::ability::TriggeredAbility;
use common::ability::ZoneMoveTrigger;
use common::card_entities::CardEnt;
use common::cost::Cost;
use common::entities::PlayerId;
use common::mana::ManaCostSymbol;
use common::spellabil::Clause;
use common::spellabil::Constraint;
use common::spellabil::KeywordAbility;
use common::zones::Zone;
use log::debug;
use log::info;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
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
use serde_with::{serde_as, BorrowCow};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use texttoken::owned_tokens;
use texttoken::{tokens, Token, Tokens};
pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

enum ParsedLine {
    Clause(Clause),
    Abil(Ability),
}
pub struct CardDB {
    scryfall: HashMap<Token, ScryfallEntry>,
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
#[serde_as]
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ScryfallEntry {
    #[serde_as(as = "Option<BorrowCow>")]
    pub object: Option<Cow<'static, str>>,
    #[serde_as(as = "BorrowCow")]
    pub name: Cow<'static, str>,
    pub image_uris: Option<ScryfallImageUrls>,
    #[serde_as(as = "Option<BorrowCow>")]
    pub mana_cost: Option<Cow<'static, str>>,
    #[serde_as(as = "Option<BorrowCow>")]
    pub type_line: Option<Cow<'static, str>>,
    pub tokenized_type_line: Option<Vec<Token>>, //Will be tokeized upon construction
    #[serde_as(as = "Option<BorrowCow>")]
    pub lang: Option<Cow<'static, str>>,
    pub color_identity: Option<Vec<Token>>,
    pub cmc: Option<f64>,
    #[serde_as(as = "Option<BorrowCow>")]
    pub power: Option<Cow<'static, str>>,
    #[serde_as(as = "Option<BorrowCow>")]
    pub toughness: Option<Cow<'static, str>>,
    #[serde_as(as = "Option<BorrowCow>")]
    pub oracle_text: Option<Cow<'static, str>>,
    pub tokenized_oracle_text: Option<Vec<Token>>, //Will be tokeized upon construction
}
pub fn nom_error<'a>(
    tokens: &'a Tokens,
    message: &'static str,
) -> nom::Err<VerboseError<&'a Tokens>> {
    nom::Err::Error(VerboseError {
        errors: vec![(tokens, nom::error::VerboseErrorKind::Context(message))],
    })
}

fn find_path() -> Result<PathBuf, std::io::Error> {
    //let path = "../oracle-cards-20230120100202.json";
    let current_dir = std::env::current_dir()?;
    let dir_copy = current_dir.clone();
    let parent_dir = dir_copy.parent().unwrap();
    for entry in fs::read_dir(current_dir)?.chain(fs::read_dir(parent_dir)?) {
        let entry = entry?;
        let path = entry.path();
        let last = path.file_stem();
        if let Some(last) = last {
            if let Some(last) = last.to_str() {
                if last.contains("oracle-cards-") {
                    return Ok(path);
                }
            }
        }
    }
    panic!("Failed to find scryfall oracle database");
}
impl CardDB {
    pub fn new() -> Self {
        println!("Initializing card database");
        let path = find_path().expect("Failed to find scryfall oracle database");
        let data = fs::read_to_string(path).expect("Couldn't open file");
        let data: &'static str = Box::leak(data.into_boxed_str());
        let desered: Vec<ScryfallEntry> = serde_json::from_str(&data).expect("failed to parse!");
        let mut byname = HashMap::new();
        for mut card in desered {
            card.tokenized_type_line =
                card.type_line
                    .as_ref()
                    .map(|line: &Cow<'static, str>| match line {
                        Cow::Borrowed(line) => tokenize(line, None),
                        Cow::Owned(line) => tokenize(line, None)
                            .into_iter()
                            .map(|item| item.into_owned().into())
                            .collect(),
                    });
            card.tokenized_oracle_text = card.oracle_text.as_ref().map(|line| match line {
                Cow::Borrowed(line) => tokenize(line, Some(&card.name)),
                Cow::Owned(line) => tokenize(line, Some(&card.name))
                    .into_iter()
                    .map(|item| item.into_owned().into())
                    .collect(),
            });
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
        card.name = card_name.to_string();
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
    many0(parse_keyword_abil_and_comma)(tokens)
}
fn parse_keyword_abil_and_comma<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, KeywordAbility> {
    let (tokens, keyword) = parse_keyword_ability(tokens)?;
    let (tokens, _) = opt(tag(tokens![","]))(tokens)?;
    let (tokens, _) = opt(tag(tokens!["\n"]))(tokens)?;
    Ok((tokens, keyword))
}
//This parses all keywords that are just a phrase with no additional info
fn parse_keyword_ability<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, KeywordAbility> {
    let basics = vec![
        (
            owned_tokens!["first", "strike"],
            KeywordAbility::FirstStrike,
        ),
        (
            owned_tokens!["double", "strike"],
            KeywordAbility::DoubleStrike,
        ),
        (owned_tokens!["flying"], KeywordAbility::Flying),
        (owned_tokens!["lifelink"], KeywordAbility::Lifelink),
        (owned_tokens!["vigilance"], KeywordAbility::Vigilance),
        (owned_tokens!["trample"], KeywordAbility::Trample),
        (owned_tokens!["prowess"], KeywordAbility::Prowess),
        (owned_tokens!["flash"], KeywordAbility::Flash),
    ];
    for (text, abil) in basics {
        if let Ok((tokens, _)) = (tag::<_, _, VerboseError<_>>(Tokens::from_array(&text)))(tokens) {
            return Ok((tokens, abil));
        }
    }
    Err(nom_error(tokens, "failed to parse basic keyword"))
}

fn prune_comment<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ()> {
    let (tokens, _) = opt(delimited(
        tag(tokens!["("]),
        is_not(tokens!(")")),
        tag(tokens!(")")),
    ))(tokens)?;
    let (tokens, _) = opt(tag(tokens!["\n"]))(tokens)?;
    Ok((tokens, ()))
}
fn parse_mana_symbol_inner<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<ManaCostSymbol>> {
    let (tokens, first) = take(1 as usize)(tokens)?;
    let first = &*first[0];
    let mut res = vec![];
    if first == "w" {
        res = vec![ManaCostSymbol::White];
    }
    if first == "u" {
        res = vec![ManaCostSymbol::Blue];
    }
    if first == "b" {
        res = vec![ManaCostSymbol::Black];
    }
    if first == "r" {
        res = vec![ManaCostSymbol::Red];
    }
    if first == "g" {
        res = vec![ManaCostSymbol::Green];
    }
    if let Ok(num) = i64::from_str(first) {
        res = vec![
            ManaCostSymbol::Generic;
            num.try_into()
                .map_err(|_| nom_error(tokens, "not a positive integer"))?
        ];
    }
    Ok((tokens, res))
}
fn parse_mana_symbol<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<ManaCostSymbol>> {
    let (tokens, _) = tag(tokens!["{"])(tokens)?;
    let (tokens, res) = parse_mana_symbol_inner(tokens)?;
    let (tokens, _) = tag(tokens!["}"])(tokens)?;
    Ok((tokens, res))
}
fn parse_mana_cost<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<Cost>> {
    let (tokens, manas) = many1(parse_mana_symbol)(tokens)?;
    let manas = manas.into_iter().flatten().map(|x| Cost::Mana(x)).collect();
    Ok((tokens, manas))
}
fn parse_tap_cost<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<Cost>> {
    let (tokens, _) = tag(tokens!["{", "t", "}"])(tokens)?;
    Ok((tokens, vec![Cost::Selftap]))
}

fn parse_tap_comma<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<Cost>> {
    let (tokens, _) = tag(tokens![","])(tokens)?;
    Ok((tokens, vec![]))
}

fn parse_cost<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<Cost>> {
    alt((parse_mana_cost, parse_tap_cost, parse_tap_comma))(tokens)
}

fn parse_costs<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<Cost>> {
    let (tokens, manas) = many1(parse_cost)(tokens)?;
    let costs = manas.into_iter().flatten().collect();
    Ok((tokens, costs))
}
fn parse_activate_only<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    fn if_you_control<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
        let (tokens, _) = tag(tokens!["if", "you", "control"])(tokens)?;
        let (tokens, num) = parse_number(tokens)?;
        let (tokens, r) = many1(parse_constraint)(tokens)?;
        Ok((tokens, Constraint::ControlWith(r, num)))
    }
    let (tokens, _) = tag(tokens![".", "activate", "only"])(tokens)?;
    let (tokens, res) = alt((if_you_control,))(tokens)?;
    Ok((tokens, res))
}
fn parse_activated_abil<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Ability> {
    let (tokens, costs) = parse_costs(tokens)?;
    let (tokens, _) = tag(tokens![":"])(tokens)?;
    let (tokens, clauses) = many1(parse_clause)(tokens)?;
    let (tokens, activate_only) = opt(parse_activate_only)(tokens)?;
    Ok((
        tokens,
        Ability::Activated(ActivatedAbility {
            costs,
            effect: clauses,
            keyword: None,
            restrictions: activate_only,
        }),
    ))
}

fn parse_etb_trigger<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, AbilityTrigger> {
    let (tokens, constraint) = many1(parse_constraint)(tokens)?;
    let (tokens, _) = tag(tokens!["enter", "the", "battlefield"])(tokens)?;
    Ok((
        tokens,
        AbilityTrigger {
            trigger: AbilityTriggerType::ZoneMove(ZoneMoveTrigger {
                origin: None,
                dest: Some(Zone::Battlefield),
            }),
            constraint,
        },
    ))
}

fn parse_death_trigger<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, AbilityTrigger> {
    let (tokens, constraint) = many1(parse_constraint)(tokens)?;
    let (tokens, _) = tag(tokens!["die"])(tokens)?;
    Ok((
        tokens,
        AbilityTrigger {
            trigger: AbilityTriggerType::ZoneMove(ZoneMoveTrigger {
                origin: Some(Zone::Battlefield),
                dest: Some(Zone::Graveyard),
            }),
            constraint,
        },
    ))
}
fn parse_ability_trigger<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, AbilityTrigger> {
    alt((parse_etb_trigger, parse_death_trigger))(tokens)
}
fn parse_comma_if_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens![",", "if"])(tokens)?;
    let (tokens, constraint) = parse_constraint(tokens)?;
    Ok((tokens, constraint))
}
fn parse_triggered_ability<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Ability> {
    let (tokens, _) = alt((tag(tokens!["when"]), tag(tokens!["whenever"])))(tokens)?;
    let (tokens, mut trigger) = parse_ability_trigger(tokens)?;
    let (tokens, if_clause) = opt(parse_comma_if_clause)(tokens)?;
    if let Some(constraint) = if_clause {
        trigger.constraint.push(constraint);
    }
    let (tokens, _) = tag(tokens![","])(tokens)?;
    let (tokens, effect) = many1(parse_clause)(tokens)?;
    Ok((
        tokens,
        Ability::Triggered(TriggeredAbility {
            trigger,
            effect,
            keyword: None,
        }),
    ))
}
fn parse_protection_from_x_and_from_y<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, StaticAbility> {
    let (tokens, _) = tag(tokens!["protection", "from"])(tokens)?;
    let (tokens, type1) = parse_constraint(tokens)?;
    let (tokens, _) = tag(tokens!["and", "from"])(tokens)?;
    let (tokens, type2) = parse_constraint(tokens)?;
    Ok((
        tokens,
        StaticAbility {
            keyword: Some(KeywordAbility::Protection),
            effect: StaticAbilityEffect::Protection(Constraint::Or(vec![type1, type2])),
        },
    ))
}
fn parse_protection_from<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, StaticAbility> {
    let (tokens, _) = tag(tokens!["protection", "from"])(tokens)?;
    let (tokens, constraint) = parse_constraint(tokens)?;
    Ok((
        tokens,
        StaticAbility {
            keyword: Some(KeywordAbility::Protection),
            effect: StaticAbilityEffect::Protection(constraint),
        },
    ))
}

fn parse_zonemove_replacement<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ReplacementAbility> {
    fn parse_zone_move<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ZoneMoveTrigger> {
        fn parse_etb<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ZoneMoveTrigger> {
            let (tokens, _) = tag(tokens!["enter", "the", "battlefield"])(tokens)?;
            Ok((
                tokens,
                ZoneMoveTrigger {
                    origin: None,
                    dest: Some(Zone::Battlefield),
                },
            ))
        }
        alt((parse_etb,))(tokens)
    }
    fn parse_and_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
        let (tokens, _) = tag(tokens!["and", "it"])(tokens)?;
        parse_constraint(tokens)
    }
    let (tokens, _) = tag(tokens!["if"])(tokens)?;
    let (tokens, _) = opt(tag(tokens!["a"]))(tokens)?;
    let (tokens, mut constraints) = many1(parse_constraint)(tokens)?;
    let (tokens, _) = opt(tag(tokens!["would"]))(tokens)?;
    let (tokens, trigger) = parse_zone_move(tokens)?;
    let (tokens, add_constraint) = opt(parse_and_clause)(tokens)?;
    add_constraint.map(|x|constraints.push(x));
    let (tokens, _) = tag(tokens![","])(tokens)?;
    let (tokens,new_effect)=parse_clause(tokens)?;
    let (tokens, _) = tag(tokens!["instead"])(tokens)?;
    Ok((tokens,ReplacementAbility{
        keyword:None,
        effect: Replacement::ZoneMoveReplacement { trigger, new_effect, constraints }
    }))
}
fn parse_replacement_abil<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Ability> {
    let (tokens, abil) = alt((parse_zonemove_replacement,))(tokens)?;
    Ok((tokens, Ability::Replacement(abil)))
}
fn parse_static_abil<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Ability> {
    let (tokens, abil) = alt((parse_protection_from_x_and_from_y, parse_protection_from))(tokens)?;
    Ok((tokens, Ability::Static(abil)))
}
fn parse_abil<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Ability> {
    alt((
        parse_activated_abil,
        parse_triggered_ability,
        parse_static_abil,
        parse_replacement_abil,
    ))(tokens)
}
fn parse_clause_or_abil<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ParsedLine> {
    let attempt_clause = parse_clause(tokens);
    if let Ok((tokens, clause)) = attempt_clause {
        return Ok((tokens, ParsedLine::Clause(clause)));
    }
    let (tokens, abil) = parse_abil(tokens)?;
    return Ok((tokens, ParsedLine::Abil(abil)));
}

fn parse_body_lines<'a>(card: &mut CardEnt, tokens: &'a Tokens) -> Res<&'a Tokens, ()> {
    let (mut tokens, keywords) = context("parse keywords", parse_keyword_abilities)(tokens)?;
    for keyword in keywords {
        card.abilities.push(Ability::from_keyword(keyword));
    }
    while tokens.len() > 0 {
        let parsedline;
        (tokens, _) = context("pruning comments", prune_comment)(tokens)?;
        if tokens.len() == 0 {
            break;
        }
        (tokens, parsedline) = parse_clause_or_abil(tokens)?;
        match parsedline {
            ParsedLine::Clause(clause) => {
                card.effect.push(clause);
            }
            ParsedLine::Abil(abil) => {
                card.abilities.push(abil);
            }
        }
        (tokens, _) = opt(tag(tokens!(".")))(tokens)?;
        (tokens, _) = opt(tag(tokens!("\n")))(tokens)?;
    }
    let (tokens, _) = nom::combinator::opt(tag(tokens!["\n"]))(tokens)?;
    Ok((tokens, ()))
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
