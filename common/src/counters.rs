use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq, Copy)]
pub enum Counter {
    Plus1Plus1,
}
