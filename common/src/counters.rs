use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use mtg_log_macro::MTGLoggable;
use crate::log::{MTGLog,GameContext};

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq, Copy, MTGLoggable)]
pub enum Counter {
    Plus1Plus1,
}
