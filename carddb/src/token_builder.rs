use std::str::FromStr;

use crate::carddb::{nom_error, parse_number, Res};
use cardtypes::{Subtype, Type};
use common::{ability::Ability, card_entities::PT, spellabil::KeywordAbility};
use common::{mana::Color, token_attribute::TokenAttribute};
use nom::bytes::complete::tag;
use nom::{branch::alt, multi::many0};
use texttoken::tokens;
use texttoken::Tokens;
fn parse_pt<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, TokenAttribute> {
    let (tokens, power) = parse_number(tokens)?;
    let (tokens, _) = tag(tokens!["/"])(tokens)?;
    let (tokens, toughness) = parse_number(tokens)?;
    Ok((tokens, TokenAttribute::PT(PT { power, toughness })))
}
fn parse_hascolor<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, TokenAttribute> {
    if tokens.len()>0 && let Ok(color)=Color::from_str(&tokens[0]){
    Ok((&tokens[1..],TokenAttribute::HasColor(color)))
  }else{
    Err(nom_error(tokens, "failed to parse color"))
  }
}
fn parse_type<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, TokenAttribute> {
    if tokens.len()>0 && let Ok(t)=Type::from_str(&tokens[0]){
    Ok((&tokens[1..],TokenAttribute::Type(t)))
  }else{
    Err(nom_error(tokens, "failed to parse type"))
  }
}
fn parse_subtype<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, TokenAttribute> {
    if tokens.len()>0 && let Ok(t)=Subtype::from_str(&tokens[0]){
    Ok((&tokens[1..],TokenAttribute::Subtype(t)))
  }else{
    Err(nom_error(tokens, "failed to parse type"))
  }
}
fn parse_keyword_ability<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, TokenAttribute> {
    if tokens.len()>0 && let Ok(abil)=KeywordAbility::from_str(&tokens[0]){
    Ok((&tokens[1..],TokenAttribute::Ability(Ability::from_keyword(abil))))
  }else{
    Err(nom_error(tokens, "failed to parse type"))
  }
}
fn parse_token_attribute<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, TokenAttribute> {
    alt((
        parse_pt,
        parse_hascolor,
        parse_type,
        parse_subtype,
        parse_keyword_ability,
    ))(tokens)
}
pub fn parse_token_attributes<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Vec<TokenAttribute>> {
    many0(parse_token_attribute)(tokens)
}
