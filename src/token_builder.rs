use std::str::FromStr;

use crate::tokens;
use crate::{
    ability::Ability,
    card_entities::PT,
    card_types::{Subtype, Type},
    carddb::{nom_error, parse_number, text_token::Tokens, Res},
    mana::Color,
    spellabil::KeywordAbility,
};
use nom::bytes::complete::tag;
use nom::{branch::alt, multi::many0};
use schemars::JsonSchema;
use serde::Serialize;
#[derive(Clone, Serialize, JsonSchema, Debug)]
pub enum TokenAttribute {
    PT(PT),
    HasColor(Color),
    Type(Type),
    Subtype(Subtype),
    Ability(Ability),
}
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
