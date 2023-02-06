use std::borrow::Cow;
use crate::carddb::{nom_error, Res};
use std::str::FromStr;
use nom::{bytes::complete::tag, error::VerboseError};
use texttoken::{Tokens, owned_tokens};

fn to_int(number: &str) -> Option<i64> {
    if let Ok(num) = i64::from_str(number) {
        return Some(num);
    }
    let first = number.chars().next()?;
    let sign = match first {
        '+' => 1,
        '-' => -1,
        _ => {
            return None;
        }
    };
    Some(sign * i64::from_str(&number[1..]).ok()?)
}
pub fn parse_number<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, i64> {
    if tokens.len() == 0 {
        return Err(nom_error(tokens, "Empty tokens when parsing integer"));
    }
    let words = vec![
        (owned_tokens!["a"], 1),
        (owned_tokens!["one"], 1),
        (owned_tokens!["two"], 2),
    ];
    for (text, num) in words {
        if let Ok((tokens, _)) = (tag::<_, _, VerboseError<_>>(Tokens::from_array(&text)))(tokens) {
            return Ok((tokens, num));
        }
    }
    let first_token = &tokens[0];
    if let Some(num) = to_int(&first_token) {
        Ok((&tokens[1..], num))
    } else {
        Err(nom_error(tokens, "Failed to parse integer"))
    }
}
