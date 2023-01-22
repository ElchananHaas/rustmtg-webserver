
use texttoken::{Tokens};
use std::str::FromStr;
use crate::{carddb::{Res, nom_error}};

fn to_int(number:&str) -> Option<i64>{
    if let Ok(num) = i64::from_str(number){
        return Some(num);
    }
    let first=number.chars().next()?;
    let sign=match first {
        '+'=>1,
        '-'=>-1,
        _=>{return None;}
    };
    Some(sign * i64::from_str(&number[1..]).ok()?)
}
pub fn parse_number<'a>(tokens: &'a Tokens) -> Res<&'a Tokens, i64> {
    if tokens.len() == 0 {
        return Err(nom_error(tokens, "Empty tokens when parsing integer"));
    }
    let first_token = &tokens[0];
    if let Some(num) = to_int(&first_token) {
        Ok((&tokens[1..], num))
    } else {
        Err(nom_error(tokens, "Failed to parse integer"))
    }
}
