use crate::{
    card_types::{Supertypes, Subtypes, Types},
    entities::MIN_CARDID,
    spellabil::Clause,
};
use derivative::*;
use schemars::JsonSchema;
use serde_derive::Serialize;
use std::{collections::HashSet, num::NonZeroU64};

use crate::{
    ability::Ability,
    cost::Cost,
    entities::{CardId, PlayerId, TargetId},
    spellabil::KeywordAbility,
};
#[derive(Clone, Serialize, PartialEq, Eq, JsonSchema)]
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
#[derive(Serialize, Clone, JsonSchema)]
//Holds a card, token or embalem, or triggered/activated ability
pub struct CardEnt {
    pub etb_this_cycle: bool,
    pub damaged: i64,
    pub tapped: bool,
    pub already_dealt_damage: bool, //Has this dealt combat damage this turn (First strike, Double strike)
    pub attacking: Option<TargetId>, //Is this attacking a player of planeswalker
    pub blocked: Vec<CardId>,   //What creatues is this blocked by?
    pub blocking: Vec<CardId>,
    pub effect: Vec<Clause>, //Effect of card, for instant sorcery or ability
    pub name: &'static str,
    #[derivative(Default(value = "PlayerId::from(NonZeroU64::new(MIN_CARDID-1).unwrap())"))]
    pub owner: PlayerId,
    pub printed_name: &'static str,
    pub ent_type: EntType,
    pub known_to: HashSet<PlayerId>, //What players know the front side of this card?
    pub pt: Option<PT>,
    pub controller: Option<PlayerId>,
    pub types: Types,
    pub supertypes: Supertypes,
    pub subtypes: Subtypes,
    pub abilities: Vec<Ability>,
    pub costs: Vec<Cost>, //Casting costs
    pub art_url: Option<String>,
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
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
pub struct PT {
    pub power: i64,
    pub toughness: i64,
}
