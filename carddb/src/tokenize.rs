use std::{borrow::Cow, str::FromStr};

use common::cardtypes::Subtype;
use common::spellabil::KeywordAbility;
use nom::{
    bytes::complete::{is_not, take},
    multi::many0,
    IResult,
};

//Lowercase the word and trim trailing s.
//Don't trim trailing s for subtypes and keywords
fn lemmatize<'a>(word: &'a str) -> Cow<'a, str> {
    let word: Cow<'a, str> = if word.chars().all(|char| char.is_lowercase()) {
        word.into()
    } else {
        word.to_lowercase().into()
    };
    if word.chars().last() != Some('s') {
        return word.into();
    }
    let dont_trim = ["its", "this", "has", "is", "unless"];
    if dont_trim.into_iter().any(|x| x == word) {
        return word.into();
    }
    if Subtype::from_str(&word).is_ok() || KeywordAbility::from_str(&word).is_ok() {
        return word;
    }
    let mut word = word.to_string();
    let dont_ies = ["dies"];
    if word.chars().rev().take(3).collect::<Vec<_>>() == vec!['s', 'e', 'i']
        && (!dont_ies.iter().any(|&x| x == &word))
    {
        for _ in 0..3 {
            word.pop();
        }
        word.push('y');
    } else {
        word.pop();
    }
    return word.into();
}
fn parse_token<'a>(mut text: &'a str) -> IResult<&'a str, Cow<'a, str>, ()> {
    let special_chars = " .:,\"\n()/{}";
    (text, _) = many0(nom::character::complete::char(' '))(text)?;
    if text.len() == 0 {
        return Err(nom::Err::Error(()));
    }
    {
        let (rest, first) = take(1usize)(text)?;
        if special_chars.contains(first) {
            return Ok((rest, first.into()));
        }
    }
    let (rest, word) = is_not::<_, _, ()>(special_chars)(text).expect("found non special chars");
    if word.len() > 0 {
        return Ok((rest, lemmatize(word)));
    }
    return Err(nom::Err::Error(()));
}

pub fn tokenize<'a, 'b>(text: &'a str, name: Option<&'b str>) -> Vec<Cow<'a, str>> {
    //TODO replace name with cardname properly
    let mut parts: Vec<&'a str> = Vec::new();
    if let Some(name) = name {
        for chunk in text.split(name) {
            parts.push(chunk);
        }
    } else {
        parts.push(text);
    }
    let mut res = Vec::new();
    for part in parts {
        let (_, mut parsed) = many0(parse_token)(&part).expect("Tokenizing failed");
        res.append(&mut parsed);
        res.push("cardname".into())
    }
    res.pop();
    res
}
