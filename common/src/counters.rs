use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize,  Deserialize, JsonSchema, Debug, PartialEq, Copy)]
pub enum Counter {
    Plus1Plus1,
}
