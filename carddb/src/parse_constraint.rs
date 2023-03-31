use std::str::FromStr;

use common::{
    cardtypes::{ParseType, Subtype, Type},
    spellabil::{Constraint, KeywordAbility},
};
use nom::combinator::opt;
use nom::{branch::alt, bytes::complete::take};
use nom::{bytes::complete::tag, multi::many1};
use texttoken::{tokens, Tokens};

use crate::{
    carddb::{nom_error, Res},
    parse_clauseeffect::parse_counter,
};

pub fn parse_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let type_constraint = nom::combinator::map(Type::parse, |t| Constraint::CardType(t));
    let (tokens, constraint) = alt((
        parse_tapped_constraint,
        parse_other_constraint,
        type_constraint,
        parse_cardname_constraint,
        parse_you_control_constraint,
        parse_keyword_constraint,
        parse_subtype_constraint,
        parse_has_counter,
        parse_multicolored_constraint,
        parse_nontoken_constraint,
        parse_not_cast,
    ))(tokens)?;
    let (tokens, or_part) = opt(parse_or_constraint)(tokens)?;
    if let Some(or_part) = or_part {
        Ok((tokens, Constraint::Or(vec![constraint, or_part])))
    } else {
        Ok((tokens, constraint))
    }
}
fn parse_not_cast<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["wasn't", "cast"])(tokens)?;
    Ok((tokens, Constraint::NotCast))
}
fn parse_multicolored_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["multicolored"])(tokens)?;
    Ok((tokens, Constraint::Multicolored))
}

fn parse_nontoken_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["nontoken"])(tokens)?;
    Ok((tokens, Constraint::NonToken))
}

fn parse_or_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["or"])(tokens)?;
    let (tokens, constraint) = many1(parse_constraint)(tokens)?;
    Ok((tokens, Constraint::And(constraint)))
}

fn parse_tapped_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["tapped"])(tokens)?;
    Ok((tokens, Constraint::IsTapped))
}

fn parse_cardname_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["cardname"])(tokens)?;
    Ok((tokens, Constraint::IsCardname))
}

fn parse_other_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = alt((tag(tokens!["another"]), tag(tokens!["other"])))(tokens)?;
    Ok((tokens, Constraint::Other))
}

fn parse_you_control_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["you", "control"])(tokens)?;
    Ok((tokens, Constraint::YouControl))
}

fn parse_keyword_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["with"])(tokens)?;
    let (tokens, first) = take(1usize)(tokens)?;
    let abil = KeywordAbility::from_str(&*first[0])
        .map_err(|_| nom_error(tokens, "failed to parse keyword ability"))?;
    Ok((tokens, Constraint::HasKeyword(abil)))
}

fn parse_subtype_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, subtype) = Subtype::parse(tokens)?;
    Ok((tokens, Constraint::Subtype(subtype)))
}

fn parse_has_counter<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Constraint> {
    let (tokens, _) = tag(tokens!["it"])(tokens)?;
    let (tokens, _) = alt((tag(tokens!["had"]), tag(tokens!["has"])))(tokens)?;
    let (tokens, _) = tag(tokens!["a"])(tokens)?;
    let (tokens, counter) = parse_counter(tokens)?;
    let (tokens, _) = tag(tokens!["counter"])(tokens)?;
    let (tokens, _) = opt(tag(tokens!["on", "it"]))(tokens)?;
    Ok((tokens, Constraint::HasCounter(counter)))
}
