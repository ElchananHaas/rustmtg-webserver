use schemars::JsonSchema;
use serde::Serialize;

#[derive(Clone, Serialize, JsonSchema, Debug, PartialEq, Copy)]
pub enum Counter{
    Plus1Plus1
}