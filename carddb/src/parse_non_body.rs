use cardtypes::{Supertypes, Types, Subtypes};
use common::{card_entities::{CardEnt, PT}, cost::Cost};
use nom::{error::VerboseError, bytes::complete::tag, combinator::opt};
use texttoken::Tokens;

use crate::carddb::{ScryfallEntry, parse_mana, Res, nom_error};

pub fn parse_pt<'a>(card: &mut CardEnt, entry: &'a ScryfallEntry) {
    if let Some(power)=entry.power.as_ref()
    && let Some(toughness)=entry.toughness.as_ref(){
        let res;
        if let Ok(power)=power.parse::<i64>()
        && let Ok(toughness)=toughness.parse::<i64>(){
            res=(power,toughness)
        }else{
            res=(0,0)
        }
        card.pt = Some(PT {
            power: res.0,
            toughness: res.1,
        });
    };
}


pub fn parse_cost_line<'a>(
    card: &mut CardEnt,
    entry: &'a ScryfallEntry,
) -> Result<(), nom::Err<VerboseError<&'a str>>> {
    if let Some(manatext) = entry.mana_cost.as_ref() {
        let (rest, manas) = parse_mana(&manatext)?;
        if rest.len() > 0 {
            panic!("parser error!");
        }
        for mana in manas {
            card.costs.push(Cost::Mana(mana));
        }
        Ok(())
    } else {
        Err(nom::Err::Error(VerboseError {
            errors: vec![(
                "",
                nom::error::VerboseErrorKind::Context("card had no cost line"),
            )],
        }))
    }
}

pub fn parse_type_line<'a>(card: &mut CardEnt, entry: &'a ScryfallEntry) -> Res<&'a Tokens, ()> {
    if let Some(tokenized) = entry.tokenized_type_line.as_ref() {
        let tokens = Tokens::from_array(&tokenized);
        let (rest, (types, subtypes, supertypes)) = parse_type_line_h(&tokens)?;
        if rest.len() == 0 {
            card.types = types;
            card.supertypes = supertypes;
            card.subtypes = subtypes;
            Ok((rest, ()))
        } else {
            Err(nom_error(rest, "failed to parse complete type line"))
        }
    } else {
        Err(nom_error(
            &Tokens::empty(),
            "scryfall entry had no type line",
        ))
    }
}
fn parse_type_line_h<'a>(text: &'a Tokens) -> Res<&'a Tokens, (Types, Subtypes, Supertypes)> {
    let (text, supertypes) = Supertypes::parse(text)?;
    let (text, types) = Types::parse(text)?;
    let (text, _) = opt(tag(Tokens::from_array(&["—".to_owned()])))(text)?;
    let (text, subtypes) = Subtypes::parse(text)?;
    let (text, _) = opt(tag(Tokens::from_array(&["\n".to_owned()])))(text)?;
    Ok((text, (types, subtypes, supertypes)))
}
