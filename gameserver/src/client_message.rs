use crate::{
    actions::Action,
    card_entities::CardEnt,
    entities::{CardId, PlayerId, TargetId},
    game::{Game, Zone},
    hashset_obj::HashSetObj,
    player::PlayerView,
};
use schemars::JsonSchema;
use serde::Serialize;
use std::{collections::HashMap, hash::Hash};
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
pub struct AskPairItem<T: Hash + Eq> {
    pub items: HashSetObj<T>,
    pub min: usize, //inclusive
    pub max: usize, //inclusive
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct AskPair<T: Hash + Eq> {
    pub pairs: HashMap<CardId, AskPairItem<T>>,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub enum Ask {
    Attackers(AskPair<TargetId>),
    Blockers(AskPair<CardId>),
    DiscardToHandSize(AskSelectN<CardId>),
    Action(AskSelectN<Action>),
    Target(AskSelectN<TargetId>),
}
