use common::{
    card_entities::PT,
    cardtypes::{ParseType, Subtype},
    counters::Counter,
    spellabil::{ClauseEffect, ContEffect, NumberComputer},
};

use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::{branch::alt, multi::many1};

use texttoken::{tokens, Tokens};

use crate::{
    carddb::{parse_abil, Res},
    parse_constraint::parse_constraint,
    token_builder::parse_token_attributes,
    util::parse_number,
};

pub fn parse_action_second_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, effect) = alt((
        parse_gain_life,
        parse_draw_a_card,
        parse_create_token,
        parse_until_end_turn,
    ))(tokens)?;
    let for_clause = parse_for_clause(tokens);
    if let Ok((tokens, computer)) = for_clause {
        Ok((tokens, ClauseEffect::MultClause(Box::new(effect), computer)))
    } else {
        Ok((tokens, effect))
    }
}
fn parse_for_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, NumberComputer> {
    let (tokens, _) = tag(tokens!["for", "each"])(tokens)?;
    let (tokens, constraints) = many1(parse_constraint)(tokens)?;
    Ok((tokens, NumberComputer::NumPermanents(constraints)))
}
fn parse_gain_life<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, _) = tag(tokens!["gain"])(tokens)?;
    let (tokens, value) = parse_number(tokens)?;
    let (tokens, _) = tag(tokens!["life"])(tokens)?;
    Ok((tokens, ClauseEffect::GainLife(value)))
}
fn parse_create_token<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, _) = tag(tokens!["create", "a"])(tokens)?;
    let (tokens, attr1) = parse_token_attributes(tokens)?;
    let (tokens, _) = tag(tokens!["token"])(tokens)?;
    let (tokens, _) = opt(tag(tokens!["with"]))(tokens)?;
    let (tokens, attr2) = parse_token_attributes(tokens)?;
    let attrs = attr1.into_iter().chain(attr2.into_iter()).collect();
    Ok((tokens, ClauseEffect::CreateToken(attrs)))
}

pub fn parse_action_first_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    fn parse_destroy_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
        let (tokens, _) = tag(tokens!["destroy"])(tokens)?;
        Ok((tokens, ClauseEffect::Destroy))
    }
    fn parse_exile_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
        let (tokens, _) = tag(tokens!["exile"])(tokens)?;
        Ok((tokens, ClauseEffect::Exile))
    }
    fn parse_put_counter<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
        let (tokens, _) = tag(tokens!["put"])(tokens)?;
        let (tokens, num) = parse_number(tokens)?;
        let (tokens, counter) = parse_counter(tokens)?;
        let (tokens, _) = tag(tokens!["counter", "on"])(tokens)?;
        Ok((tokens, ClauseEffect::PutCounter(counter, num)))
    }
    fn parse_tap<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
        let (tokens, _) = tag(tokens!["tap"])(tokens)?;
        Ok((tokens, ClauseEffect::Tap))
    }
    alt((
        parse_destroy_effect,
        parse_exile_effect,
        parse_put_counter,
        parse_tap,
    ))(tokens)
}

fn parse_draw_a_card<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, _) = tag(tokens!("draw", "a", "card"))(tokens)?;
    Ok((tokens, ClauseEffect::DrawCard))
}
fn parse_p1p1_coutner<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Counter> {
    let (tokens, _) = tag(tokens!("+1", "/", "+1"))(tokens)?;
    Ok((tokens, Counter::Plus1Plus1))
}
pub fn parse_counter<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Counter> {
    alt((parse_p1p1_coutner,))(tokens)
}

fn parse_until_end_turn<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ClauseEffect> {
    let (tokens, effect) = parse_cont_effect(tokens)?;
    let (tokens, _) = tag(tokens!["until", "end", "of", "turn"])(tokens)?;
    Ok((tokens, ClauseEffect::UntilEndTurn(effect)))
}

pub fn parse_cont_effect<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ContEffect> {
    fn parse_cant_attack_or_block<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ContEffect> {
        let (tokens, _) = tag(tokens!["can't", "attack", "or", "block"])(tokens)?;
        Ok((tokens, ContEffect::CantAttackOrBlock))
    }
    fn parse_cant_activate_non_mana_abils<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ContEffect> {
        let (tokens, _) = tag(tokens![
            "its",
            "activated",
            "ability",
            "can't",
            "be",
            "activated",
            "unless",
            "they're",
            "mana",
            "ability"
        ])(tokens)?;
        Ok((tokens, ContEffect::CantActivateNonManaAbil))
    }
    fn parse_pt_modification<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ContEffect> {
        let (tokens, _) = tag(tokens!["get"])(tokens)?;
        let (tokens, power) = parse_number(tokens)?;
        let (tokens, _) = tag(tokens!["/"])(tokens)?;
        let (tokens, toughness) = parse_number(tokens)?;
        Ok((tokens, ContEffect::ModifyPT(PT { power, toughness })))
    }
    fn parse_has_abil<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ContEffect> {
        let (tokens, _) = tag(tokens!["has"])(tokens)?;
        let (tokens, abil) = parse_abil(tokens)?;
        Ok((tokens, ContEffect::HasAbility(Box::new(abil))))
    }

    fn parse_add_subtypes<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, ContEffect> {
        let (tokens, _) = tag(tokens!["is", "a"])(tokens)?;
        let (tokens, subtype) = Subtype::parse(tokens)?;
        let (tokens, _) =
            opt(tag(tokens!["in", "addition", "to", "its", "other", "type"]))(tokens)?;
        Ok((tokens, ContEffect::AddSubtype(vec![subtype])))
    }

    alt((
        parse_pt_modification,
        parse_has_abil,
        parse_add_subtypes,
        parse_cant_attack_or_block,
        parse_cant_activate_non_mana_abils,
    ))(tokens)
}
