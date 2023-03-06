use nom::bytes::complete::tag;
use nom::character::complete;
use nom::combinator::opt;
use nom::error::{ErrorKind, VerboseError};
use nom::multi::many0;

use crate::card_entities::CardEnt;
use crate::card_types::{Subtypes, Supertypes, Types};
use crate::carddb::PT;
use crate::cost::Cost;
use crate::mana::ManaCostSymbol;

use super::text_token::Tokens;
use super::{nom_error, Res, ScryfallEntry};

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

fn parse_type_line_h<'a>(text: &'a Tokens) -> Res<&'a Tokens, (Types, Subtypes, Supertypes)> {
    let (text, supertypes) = Supertypes::parse(text)?;
    let (text, types) = Types::parse(text)?;
    let (text, _) = opt(tag(Tokens::from_array(&["—".to_owned()])))(text)?;
    let (text, subtypes) = Subtypes::parse(text)?;
    let (text, _) = opt(tag(Tokens::from_array(&["\n".to_owned()])))(text)?;
    Ok((text, (types, subtypes, supertypes)))
}

fn parse_manasymbol_contents(input: &str) -> Res<&str, Vec<ManaCostSymbol>> {
    if let Ok((rest, symbol)) = complete::one_of::<_, _, (&str, ErrorKind)>("WUBRG")(input) {
        let costsymbol = match symbol {
            'W' => vec![ManaCostSymbol::White],
            'U' => vec![ManaCostSymbol::Blue],
            'B' => vec![ManaCostSymbol::Black],
            'R' => vec![ManaCostSymbol::Red],
            'G' => vec![ManaCostSymbol::Green],
            _ => unreachable!("Already checked symbol"),
        };
        Ok((rest, costsymbol))
    } else {
        complete::u64(input).map(|(rest, x)| (rest, vec![ManaCostSymbol::Generic; x as usize]))
    }
}
fn parse_manasymbol(input: &str) -> Res<&str, Vec<ManaCostSymbol>> {
    nom::sequence::delimited(
        complete::char('{'),
        parse_manasymbol_contents,
        complete::char('}'),
    )(input)
}

pub fn parse_mana(input: &str) -> Res<&str, Vec<ManaCostSymbol>> {
    many0(parse_manasymbol)(input).map(|(tokens, x)| (tokens, x.into_iter().flatten().collect()))
}

pub fn parse_type_line<'a>(card: &mut CardEnt, entry: &'a ScryfallEntry) -> Res<&'a Tokens, ()> {
    if let Some(tokenized) = entry.tokenized_type_line.as_ref() {
        let tokens = Tokens::from_array(&tokenized);
        let (tokens, (types, subtypes, supertypes)) = parse_type_line_h(&tokens)?;
        if tokens.len() == 0 {
            card.types = types;
            card.supertypes = supertypes;
            card.subtypes = subtypes;
            Ok((tokens, ()))
        } else {
            Err(nom_error(tokens, "failed to parse complete type line"))
        }
    } else {
        Err(nom_error(
            &Tokens::empty(),
            "scryfall entry had no type line",
        ))
    }
}

pub fn parse_cost_line<'a>(
    card: &mut CardEnt,
    entry: &'a ScryfallEntry,
) -> Result<(), nom::Err<VerboseError<&'a str>>> {
    if let Some(manatext) = entry.mana_cost.as_ref() {
        let (tokens, manas) = parse_mana(&manatext)?;
        if tokens.len() > 0 {
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
