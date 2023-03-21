use crate::carddb::Res;
use crate::parse_clauseeffect::parse_action_first_effect;
use crate::parse_clauseeffect::parse_action_second_effect;
use crate::parse_constraint::parse_constraint;
use crate::util::parse_number;
use common::spellabil::Affected;
use common::spellabil::Clause;
use common::spellabil::ClauseEffect;
use common::spellabil::PermConstraint;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::error::context;
use nom::multi::many0;
use texttoken::{tokens, Tokens};

pub fn parse_affected<'a>(
    tokens: &'a Tokens,
) -> Res<&'a Tokens, (Affected, Option<PermConstraint>)> {
    fn parse_target<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, (Affected, Option<PermConstraint>)> {
        let (tokens, is_other) = opt(tag(tokens!("other")))(tokens)?;
        let (tokens, _) = tag(tokens!("target"))(tokens)?;
        Ok((
            tokens,
            (
                Affected::Target(None),
                is_other.map(|_| PermConstraint::Other),
            ),
        ))
    }
    fn parse_cardname<'a>(
        tokens: &'a Tokens,
    ) -> Res<&'a Tokens, (Affected, Option<PermConstraint>)> {
        let (tokens, _) = tag(tokens!("cardname"))(tokens)?;
        Ok((tokens, (Affected::Cardname, None)))
    }
    fn parse_up_to_target<'a>(
        tokens: &'a Tokens,
    ) -> Res<&'a Tokens, (Affected, Option<PermConstraint>)> {
        let (tokens, _) = opt(tag(tokens!["each", "of"]))(tokens)?;
        let (tokens, _) = tag(tokens!["up", "to"])(tokens)?;
        let (tokens, num) = parse_number(tokens)?;
        let (tokens, is_other) = opt(tag(tokens!("other")))(tokens)?;
        let (tokens, _) = tag(tokens!["target"])(tokens)?;
        let res = (
            Affected::UpToXTarget(num, vec![]),
            is_other.map(|_| PermConstraint::Other),
        );
        Ok((tokens, res))
    }
    fn parse_each<'a>(
        tokens: &'a Tokens,
    ) -> Res<&'a Tokens, (Affected, Option<PermConstraint>)> {
        let (tokens, _) = tag(tokens!["each"])(tokens)?;
        Ok((tokens, (Affected::All, None)))
    }
    alt((parse_target, parse_cardname, parse_up_to_target,parse_each))(tokens)
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
    let (tokens, (affected, other)) = parse_affected(tokens)?;
    let (tokens, mut constraints) = many0(parse_constraint)(tokens)?;
    if let Some(other) = other {
        constraints.push(other);
    }
    let (tokens, effect) = context("parsing target line", parse_action_second_effect)(tokens)?;
    let clause = Clause {
        effect,
        affected,
        constraints,
    };
    Ok((tokens, clause))
}

fn parse_action_affected_line<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, Clause> {
    let (tokens, effect) = context("parsing target line", parse_action_first_effect)(tokens)?;
    let (tokens, (affected, other)) = parse_affected(tokens)?;
    let (tokens, mut constraints) = many0(parse_constraint)(tokens)?;
    if let Some(other) = other {
        constraints.push(other);
    }
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
