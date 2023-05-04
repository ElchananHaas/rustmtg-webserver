use crate::{game::Game, player::PlayerView};
use common::{
    actions::Action,
    card_entities::CardEnt,
    entities::{CardId, PlayerId, TargetId},
    hashset_obj::HashSetObj,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameState {
    pub player: PlayerId,
    pub cards: HashMap<CardId, CardEnt>,
    pub players: HashMap<PlayerId, PlayerView>,
    #[serde(flatten)]
    pub game: Game,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum ClientMessage {
    GameState(GameState),
    AskUser(Ask),
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AskSelectN<T> {
    pub ents: Vec<T>,
    pub min: i64,
    pub max: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AskPairItem<T: Hash + Eq> {
    pub items: HashSetObj<T>,
    pub min: usize, //inclusive
    pub max: usize, //inclusive
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AskPair<T: Hash + Eq> {
    pub pairs: HashMap<CardId, AskPairItem<T>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Ask {
    Attackers(AskPair<TargetId>),
    Blockers(AskPair<CardId>),
    DiscardToHandSize(AskSelectN<CardId>),
    Action(AskSelectN<Action>),
    Target(AskSelectN<TargetId>),
}
