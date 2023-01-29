use crate::carddb::Res;
use crate::parse_clauseeffect::parse_action_first_effect;
use crate::parse_clauseeffect::parse_action_second_effect;
use crate::parse_constraint::parse_constraint;
use common::spellabil::Affected;
use common::spellabil::Clause;
use common::spellabil::ClauseEffect;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::error::context;
use nom::multi::many0;
use nom::multi::many1;
use texttoken::{tokens, Tokens};

pub fn parse_affected<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Affected> {
    fn parse_target<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Affected> {
        let (tokens, _) = tag(tokens!("target"))(tokens)?;
        Ok((tokens, Affected::Target(None)))
    }
    fn parse_cardname<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Affected> {
        let (tokens, _) = tag(tokens!("cardname"))(tokens)?;
        Ok((tokens, Affected::Cardname))
    }
    alt((parse_target, parse_cardname))(tokens)
}
pub fn parse_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, clause) = context(
        "parsing body line",
        alt((
            parse_action_affected_line,
            parse_affected_action_line,
            parse_you_clause,
        )),
    )(tokens)?;
    //let (tokens, _) = opt(tag(tokens!(".")))(tokens)?;
    //let (tokens, _) = opt(tag(tokens!("\n")))(tokens)?;
    Ok((tokens, clause))
}
fn parse_you_clause<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    //Sometimes MTG
    //Implicitly has a clause mean you if it is left out.
    //For example, "draw a card" vs. "you draw a card"
    let (tokens, _) = opt(tag(tokens!["you"]))(tokens)?;
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
    let (tokens, clause) = parse_clause(tokens)?;
    Ok((
        tokens,
        Clause {
            affected: Affected::Target(None),
            effect: ClauseEffect::SetTargetController(Box::new(clause)),
            constraints: vec![],
        },
    ))
}
fn parse_affected_action_line<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, affected) = parse_affected(tokens)?;
    let (tokens, constraints) = many0(parse_constraint)(tokens)?;
    let (tokens, effect) = context("parsing target line", parse_action_second_effect)(tokens)?;
    let clause = Clause {
        effect,
        affected: affected,
        constraints,
    };
    Ok((tokens, clause))
}

fn parse_action_affected_line<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, effect) = context("parsing target line", parse_action_first_effect)(tokens)?;
    let (tokens, affected) = parse_affected(tokens)?;
    let (tokens, constraints) = many0(parse_constraint)(tokens)?;
    let (tokens, addendum) = opt(parse_its_controller_clause)(tokens)?;
    let mut clause = Clause {
        effect,
        affected: affected,
        constraints,
    };
    if let Some(addendum) = addendum {
        clause.effect = ClauseEffect::Compound(vec![clause.clone(), addendum]);
    }
    Ok((tokens, clause))
}
