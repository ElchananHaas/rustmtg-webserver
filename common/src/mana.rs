use enum_map::Enum;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_derive::Serialize;
use strum::EnumString;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Enum, JsonSchema, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct Mana {
    pub color: Color,
    pub restriction: Option<ManaRestriction>,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct ManaRestriction {}

impl Mana {
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
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord, JsonSchema,
)]
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
