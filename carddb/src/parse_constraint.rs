use std::str::FromStr;

use common::{
    cardtypes::{ParseType, Subtype, Type},
    spellabil::{KeywordAbility, PermConstraint},
};
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::{branch::alt, bytes::complete::take};
use texttoken::{tokens, Tokens};

use crate::carddb::{nom_error, Res};

pub fn parse_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, PermConstraint> {
    let type_constraint = nom::combinator::map(Type::parse, |t| PermConstraint::CardType(t));
    let (tokens, constraint) = alt((
        parse_tapped_constraint,
        type_constraint,
        parse_cardname_constraint,
        parse_you_control_constraint,
        parse_keyword_constraint,
        parse_subtype_constraint,
    ))(tokens)?;
    let (tokens, or_part) = opt(parse_or_constraint)(tokens)?;
    if let Some(or_part) = or_part {
        Ok((tokens, PermConstraint::Or(vec![constraint, or_part])))
    } else {
        Ok((tokens, constraint))
    }
}
fn parse_or_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, PermConstraint> {
    let (tokens, _) = tag(tokens!["or"])(tokens)?;
    let (tokens, constraint) = parse_constraint(tokens)?;
    Ok((tokens, constraint))
}

fn parse_tapped_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, PermConstraint> {
    let (tokens, _) = tag(tokens!["tapped"])(tokens)?;
    Ok((tokens, PermConstraint::IsTapped))
}

fn parse_cardname_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, PermConstraint> {
    let (tokens, _) = tag(tokens!["cardname"])(tokens)?;
    Ok((tokens, PermConstraint::IsCardname))
}

fn parse_you_control_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, PermConstraint> {
    let (tokens, _) = tag(tokens!["you", "control"])(tokens)?;
    Ok((tokens, PermConstraint::YouControl))
}

fn parse_keyword_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, PermConstraint> {
    let (tokens, _) = tag(tokens!["with"])(tokens)?;
    let (tokens, first) = take(1usize)(tokens)?;
    let abil = KeywordAbility::from_str(&*first[0])
        .map_err(|_| nom_error(tokens, "failed to parse keyword ability"))?;
    Ok((tokens, PermConstraint::HasKeyword(abil)))
}

fn parse_subtype_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, PermConstraint> {
    let (tokens, subtype) = Subtype::parse(tokens)?;
    Ok((tokens, PermConstraint::Subtype(subtype)))
}
