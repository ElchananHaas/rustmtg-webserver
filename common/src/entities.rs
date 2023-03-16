use std::num::NonZeroU64;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};

pub trait IdDeserializer {
    fn custom_from(x: NonZeroU64) -> Self;
}
impl IdDeserializer for PlayerId {
    fn custom_from(x: NonZeroU64) -> Self {
        Self(x)
    }
}
impl IdDeserializer for CardId {
    fn custom_from(x: NonZeroU64) -> Self {
        Self(x)
    }
}
impl IdDeserializer for ManaId {
    fn custom_from(x: NonZeroU64) -> Self {
        Self(x)
    }
}
impl IdDeserializer for TargetId {
    fn custom_from(x: NonZeroU64) -> Self {
        if x.get() < MIN_CARDID {
            Self::Player(PlayerId::from(x))
        } else {
            Self::Card(CardId::from(x))
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, JsonSchema)]
pub struct PlayerId(NonZeroU64);
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, JsonSchema)]
pub struct CardId(NonZeroU64); //a reference to a card, spell token or permanent
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ManaId(NonZeroU64);
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TargetId {
    Player(PlayerId),
    Card(CardId),
}
impl serde::Serialize for TargetId {
    //Serialize them both to numbers, and disambiguate because
    //they occupy different numeric ranges on deserialization. I am doing this because
    //it is far easier to work with numbers than objects in javascript
    //becuase numbers have value semantics
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Player(player) => player.serialize(serializer),
            Self::Card(card) => card.serialize(serializer),
        }
    }
}
pub const MIN_CARDID: u64 = 256;

impl<'de> serde::Deserialize<'de> for TargetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = u64::deserialize(deserializer)?;
        let v = NonZeroU64::try_from(val)
            .map_err(|_| serde::de::Error::custom("Value didn't fit in nonzero U64"))?;
        Ok(if v.get() < MIN_CARDID {
            TargetId::Player(PlayerId::from(v))
        } else {
            TargetId::Card(CardId::from(v))
        })
    }
}

impl schemars::JsonSchema for TargetId {
    fn schema_name() -> std::string::String {
        "TargetId".to_string()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <i32 as JsonSchema>::json_schema(gen)
    }
}

impl From<NonZeroU64> for PlayerId {
    fn from(x: NonZeroU64) -> Self {
        if x.get() >= MIN_CARDID {
            panic!("only {} players are supported", MIN_CARDID);
        }
        Self(x)
    }
}

impl From<NonZeroU64> for CardId {
    fn from(x: NonZeroU64) -> Self {
        if x.get() < MIN_CARDID {
            panic!("cardids must be >= {}", MIN_CARDID);
        }
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
