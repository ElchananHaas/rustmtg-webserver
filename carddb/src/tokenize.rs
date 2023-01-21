use std::str::FromStr;

use cardtypes::Subtype;
use common::spellabil::KeywordAbility;
use nom::{bytes::complete::is_not, character::complete::one_of, multi::many0, IResult};

//Lowercase the word and trim trailing s.
//Don't trim trailing s for subtypes and keywords
fn lemmatize(word: &str) -> String {
    let mut word = word.to_lowercase();
    if word.chars().last() != Some('s') {
        return word;
    }
    let dont_trim = ["its"];
    if dont_trim.into_iter().any(|x| x == word) {
        return word;
    }
    if Subtype::from_str(&word).is_ok() || KeywordAbility::from_str(&word).is_ok() {
        return word;
    }
    word.pop();
    return word;
}
fn parse_token<'a>(mut text: &'a str) -> IResult<&str, String, ()> {
    let special_chars = " .:,\"\n()/";
    (text, _) = many0(nom::character::complete::char(' '))(text)?;
    if let Ok((rest, char)) = one_of::<_, _, ()>(special_chars)(text) {
        if char == ' ' {
            text = rest;
        } else {
            return Ok((rest, char.to_string()));
        }
    };
    let (rest, word) = is_not::<_, _, ()>(special_chars)(text)?;
    if word.len() > 0 {
        return Ok((rest, lemmatize(word)));
    }
    return Err(nom::Err::Error(()));
}
pub fn tokenize<'a>(text: &'a str, name: Option<&'a str>) -> Vec<String> {
    let text = if let Some(name) = name {
        text.replace(name, "cardname")
    } else {
        text.to_owned()
    };
    let (remainder, res) = many0(parse_token)(&text).expect("Tokenizing failed");
    assert!(remainder.len() == 0);
    return res;
}
