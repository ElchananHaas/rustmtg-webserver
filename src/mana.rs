use crate::actions::StackActionOption;
use crate::{
    ent_maps::EntMap,
    entities::{CardId, ManaId},
    game::Game,
};
use enum_map::Enum;
use schemars::JsonSchema;
use serde::ser::SerializeMap;
use serde_derive::Serialize;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Enum, JsonSchema)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}
#[derive(Clone, Serialize, JsonSchema)]
pub struct Mana {
    pub color: Color,
    pub restriction: Option<ManaRestriction>,
}
#[derive(Clone, Serialize, JsonSchema)]
pub struct ManaRestriction {}
impl ManaRestriction {
    pub fn approve(&self, game: &Game, action: &StackActionOption) -> bool {
        true //Add in restrications later
    }
}

impl Mana {
    //Use direct building for
    pub fn new(color: Color) -> Self {
        Self {
            color,
            restriction: None,
        }
    }
}

//Add support for hybrid mana later
//This ordering is significant,
//because we want to sort generic mana to the bottom for
//fulfilling with mana symbols last
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Hash, PartialOrd, Ord, JsonSchema)]
pub enum ManaCostSymbol {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
    Generic,
}
impl ManaCostSymbol {
    pub fn spendable_colors(self) -> Vec<Color> {
        match self {
            Self::White => vec![Color::White],
            Self::Blue => vec![Color::Blue],
            Self::Black => vec![Color::Black],
            Self::Red => vec![Color::Red],
            Self::Green => vec![Color::Green],
            Self::Colorless => vec![Color::Colorless],
            Self::Generic => vec![
                Color::White,
                Color::Blue,
                Color::Black,
                Color::Red,
                Color::Green,
                Color::Colorless,
            ],
        }
    }
}
pub fn mana_cost_string(coststr: &str) -> Vec<ManaCostSymbol> {
    let mut generic: u64 = 0;
    let mut res = Vec::new();
    for letter in coststr.chars() {
        if letter.is_digit(10) {
            generic *= 10;
            //This should be safe bc/ these are hardcoded within the code
            generic += u64::try_from(letter.to_digit(10).unwrap()).unwrap();
        }
        if letter == 'W' {
            res.push(ManaCostSymbol::White);
        }
        if letter == 'U' {
            res.push(ManaCostSymbol::Blue);
        }
        if letter == 'B' {
            res.push(ManaCostSymbol::Black);
        }
        if letter == 'R' {
            res.push(ManaCostSymbol::Red);
        }
        if letter == 'G' {
            res.push(ManaCostSymbol::Green);
        }
    }
    for _ in 0..generic {
        res.push(ManaCostSymbol::Generic);
    }
    res
}
