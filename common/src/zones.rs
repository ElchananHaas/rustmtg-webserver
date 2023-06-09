use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use mtg_log_macro::MTGLoggable;
use crate::log::MTGLog;
use crate::log::GameContext;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MTGLoggable)]
pub enum Zone {
    Hand,
    Library,
    Exile,
    Battlefield,
    Graveyard,
    Command,
    Stack,
}
