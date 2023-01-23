use common::{
    card_entities::PT,
    spellabil::{ClauseEffect, ContEffect},
};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;

use texttoken::{tokens, Tokens};

use crate::{carddb::Res, token_builder::parse_token_attributes, util::parse_number};

pub fn parse_action_second_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    alt((
        parse_gain_life,
        parse_draw_a_card,
        parse_create_token,
        parse_until_end_turn,
    ))(tokens)
}
fn parse_gain_life<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, _) = tag(tokens!["gain"])(tokens)?;
    let (tokens, value) = parse_number(tokens)?;
    let (tokens, _) = tag(tokens!["life"])(tokens)?;
    Ok((tokens, ClauseEffect::GainLife(value)))
}
fn parse_create_token<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, _) = (tag(tokens!["create"]))(tokens)?;
    let (tokens, _) = tag(tokens!["a"])(tokens)?;
    let (tokens, attr1) = parse_token_attributes(tokens)?;
    let (tokens, _) = tag(tokens!["token"])(tokens)?;
    let (tokens, _) = opt(tag(tokens!["with"]))(tokens)?;
    let (tokens, attr2) = parse_token_attributes(tokens)?;
    let attrs = attr1.into_iter().chain(attr2.into_iter()).collect();
    Ok((tokens, ClauseEffect::CreateToken(attrs)))
}

fn parse_destroy_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, _) = tag(tokens!["destroy"])(tokens)?;
    Ok((tokens, ClauseEffect::Destroy))
}
fn parse_exile_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, _) = tag(tokens!["exile"])(tokens)?;
    Ok((tokens, ClauseEffect::ExileBattlefield))
}
pub fn parse_action_first_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    alt((parse_destroy_effect, parse_exile_effect))(tokens)
}

fn parse_draw_a_card<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, _) = tag(tokens!("draw", "a", "card"))(tokens)?;
    Ok((tokens, ClauseEffect::DrawCard))
}

fn parse_until_end_turn<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, effect) = parse_cont_effect(tokens)?;
    let (tokens, _) = tag(tokens!["until", "end", "of", "turn"])(tokens)?;
    Ok((tokens, ClauseEffect::UntilEndTurn(effect)))
}

fn parse_cont_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ContEffect> {
    alt((parse_pt_modification,))(tokens)
}
fn parse_pt_modification<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ContEffect> {
    let (tokens, _) = tag(tokens!["get"])(tokens)?;
    let (tokens, power) = parse_number(tokens)?;
    let (tokens, _) = tag(tokens!["/"])(tokens)?;
    let (tokens, toughness) = parse_number(tokens)?;
    Ok((tokens, ContEffect::ModifyPT(PT { power, toughness })))
}
