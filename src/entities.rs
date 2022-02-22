use std::num::NonZeroU64;

use serde_derive::Serialize;


#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct PlayerId(NonZeroU64);
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct CardId(NonZeroU64);//a reference to a card, spell token or permanent
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct ManaId(NonZeroU64);

pub enum EntId{
    PlayerId,
    CardId,
}

impl From<NonZeroU64> for PlayerId{
    fn from(x:NonZeroU64) -> Self{
        Self(x)
    }
}

impl From<NonZeroU64> for CardId{
    fn from(x:NonZeroU64) -> Self{
        Self(x)
    }
}

impl From<NonZeroU64> for ManaId{
    fn from(x:NonZeroU64) -> Self{
        Self(x)
    }
}