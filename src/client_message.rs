use crate::{
    actions::Action,
    card_entities::CardEnt,
    entities::{CardId, PlayerId, TargetId},
    game::Game,
    player::PlayerView,
};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashMap;
#[derive(Serialize, JsonSchema)]
pub struct GameState<'a, 'b, 'c> {
    pub player: PlayerId,
    pub cards: HashMap<CardId, &'a CardEnt>,
    pub players: HashMap<PlayerId, PlayerView<'b>>,
    #[serde(flatten)]
    pub game: &'c Game,
}

#[derive(Serialize, JsonSchema)]
pub enum ClientMessage<'a, 'b, 'c> {
    GameState(GameState<'a, 'b, 'c>),
    AskUser(&'a Ask),
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct AskSelectN<T> {
    pub ents: Vec<T>,
    pub min: u32,
    pub max: u32,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct AskPairAB<T> {
    pub a: Vec<CardId>,
    pub b: Vec<T>,
    pub num_choices: Vec<(usize, usize)>,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub enum Ask {
    Attackers(AskPairAB<TargetId>),
    Blockers(AskPairAB<CardId>),
    DiscardToHandSize(AskSelectN<CardId>),
    Action(AskSelectN<Action>),
}
