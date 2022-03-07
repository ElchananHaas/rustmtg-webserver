use std::num::NonZeroU64;

use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct PlayerId(NonZeroU64);
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct CardId(NonZeroU64); //a reference to a card, spell token or permanent
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct ManaId(NonZeroU64);
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum TargetId {
    Player(PlayerId),
    Card(CardId),
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum EntId {
    Player(PlayerId),
    Card(CardId),
    Mana(ManaId),
}
impl From<NonZeroU64> for PlayerId {
    fn from(x: NonZeroU64) -> Self {
        Self(x)
    }
}

impl From<NonZeroU64> for CardId {
    fn from(x: NonZeroU64) -> Self {
        Self(x)
    }
}

impl From<NonZeroU64> for ManaId {
    fn from(x: NonZeroU64) -> Self {
        Self(x)
    }
}
impl From<PlayerId> for TargetId {
    fn from(x: PlayerId) -> Self {
        Self::Player(x)
    }
}

impl From<CardId> for TargetId {
    fn from(x: CardId) -> Self {
        Self::Card(x)
    }
}
