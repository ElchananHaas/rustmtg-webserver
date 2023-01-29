use schemars::JsonSchema;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, JsonSchema)]
pub enum Zone {
    Hand,
    Library,
    Exile,
    Battlefield,
    Graveyard,
    Command,
    Stack,
}
