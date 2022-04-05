use crate::{
    card_types::{Subtypes, Supertypes, Types},
    game::Game,
    spellabil::Clause,
};
use derivative::*;
use serde_derive::Serialize;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU64,
};

use crate::{
    ability::Ability,
    card_types::Subtype,
    cost::Cost,
    entities::{CardId, PlayerId, TargetId},
    spellabil::KeywordAbility,
};
#[derive(Clone, Serialize, PartialEq, Eq)]
pub enum EntType {
    RealCard,
    TokenCard,
    ActivatedAbility,
    TriggeredAbility,
}
impl Default for EntType {
    fn default() -> Self {
        Self::RealCard
    }
}
#[derive(Derivative)]
#[derivative(Default)]
#[derive(Serialize, Clone)]
pub struct CardEnt {
    //Holds a card, token or embalem
    pub summoning_sickness: bool,
    pub damaged: i64,
    pub tapped: bool,
    pub already_attacked: bool, //Has this dealt combat damage this turn (First strike, double strike)
    pub attacking: Option<TargetId>, //Is this attacking a player of planeswalker
    pub blocked: Vec<CardId>,
    pub blocking: Vec<CardId>,
    pub effect: Vec<Clause>, //Effect of card, for instant sorcery or ability
    pub name: &'static str,
    #[derivative(Default(value = "PlayerId::from(NonZeroU64::new(u64::MAX).unwrap())"))]
    pub owner: PlayerId,
    pub printed_name: &'static str,
    pub ent_type: EntType,
    pub known_to: HashSet<PlayerId>,
    pub pt: Option<PT>,
    pub controller: Option<PlayerId>,
    pub types: Types,
    pub supertypes: Supertypes,
    pub subtypes: Subtypes,
    pub abilities: Vec<Ability>,
    pub costs: Vec<Cost>,
}
impl CardEnt {
    pub fn has_keyword(&self, keyword: KeywordAbility) -> bool {
        for ability in &self.abilities {
            if ability.keyword() == Some(keyword) {
                return true;
            }
        }
        false
    }
    pub fn get_controller(&self) -> PlayerId {
        if let Some(pl) = self.controller {
            pl
        } else {
            self.owner
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PT {
    pub power: i64,
    pub toughness: i64,
}
