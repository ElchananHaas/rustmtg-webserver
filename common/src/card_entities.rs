use crate::{
    cardtypes::{Subtypes, Supertypes, Types},
    counters::Counter,
    entities::MIN_CARDID,
    hashset_obj::HashSetObj,
    mana::Color,
    spellabil::Clause,
};
use derivative::*;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_derive::Serialize;
use std::{collections::HashSet, num::NonZeroU64};

use crate::{
    ability::Ability,
    cost::Cost,
    entities::{CardId, PlayerId, TargetId},
    spellabil::KeywordAbility,
};
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema, Debug)]
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
#[derivative(Default, Debug)]
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
//Holds a card, token or embalem, or triggered/activated ability
pub struct CardEnt {
    pub etb_this_cycle: bool,
    pub damaged: i64,
    pub tapped: bool,
    pub already_dealt_damage: bool, //Has this dealt combat damage this turn (First strike, Double strike)
    pub attacking: Option<TargetId>, //Is this attacking a player of planeswalker
    pub blocked: Vec<CardId>,       //What creatues is this blocked by?
    pub blocking: Vec<CardId>,
    pub colors: HashSetObj<Color>,
    pub effect: Vec<Clause>, //Effect of card, for instant sorcery or ability
    pub name: String,
    #[derivative(Default(value = "PlayerId::from(NonZeroU64::new(MIN_CARDID-1).unwrap())"))]
    pub owner: PlayerId,
    pub ent_type: EntType,
    pub known_to: HashSet<PlayerId>, //What players know the front side of this card?
    pub pt: Option<PT>,
    controller: Option<PlayerId>,
    pub types: Types,
    pub source_of_ability: Option<CardId>, //Holds the entity
    //that produced this triggered ability if appropriate
    pub supertypes: Supertypes,
    pub subtypes: Subtypes,
    pub abilities: Vec<Ability>,
    pub costs: Vec<Cost>, //Casting costs
    pub art_url: Option<String>,
    #[derivative(Debug = "ignore")]
    pub printed: Option<Box<CardEnt>>, //This stores the printed version of the card so
    //when layers are recalculated, this can be set.
    pub counters: Vec<Counter>,
    pub cast: bool,
    pub enchanting_or_equipping: Option<TargetId>,
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
    pub fn set_controller(&mut self, controller: Option<PlayerId>) {
        self.controller = controller;
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PT {
    pub power: i64,
    pub toughness: i64,
}
