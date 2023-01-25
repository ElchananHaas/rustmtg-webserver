use common::{
    cardtypes::{ParseType, Type},
    spellabil::ClauseConstraint,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use texttoken::{tokens, Tokens};

use crate::carddb::Res;

pub fn parse_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseConstraint> {
    let type_constraint = nom::combinator::map(Type::parse, |t| ClauseConstraint::CardType(t));
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

fn parse_tapped_constraint<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseConstraint> {
    let (tokens, _) = tag(tokens!["tapped"])(tokens)?;
    Ok((tokens, ClauseConstraint::IsTapped))
}
