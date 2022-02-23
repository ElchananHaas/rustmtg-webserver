use bitvec::{array::BitArray, BitArr};
use serde_derive::Serialize;
use serde::{Serialize, Serializer, ser::SerializeSeq};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    num::NonZeroU64,
};

use crate::{
    components::Subtype,
    entities::{CardId, PlayerId, TargetId},
};

#[derive(Serialize, Clone)]
pub struct CardEnt {
    //Holds a card, token or embalem
    summoning_sickness: bool,
    pub damaged: u64,
    pub tapped: bool,
    dealt_combat_damage: bool, //Has this dealt combat damage this turn (First strike, double strike)
    pub attacking: Option<TargetId>, //Is this attacking a player of planeswalker
    pub blocked: Vec<CardId>,
    pub blocking: Vec<CardId>,
    pub name: &'static str,
    pub owner: PlayerId,
    pub printed_name: String,
    pub token: bool,
    pub known_to: HashSet<PlayerId>,
    pub pt: Option<PT>,
    pub controller: Option<PlayerId>,
    pub types: Types,
    pub supertypes: Supertypes,
    pub subtypes: Subtypes,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct PT {
    pub power: i64,
    pub toughness: i64,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Types {
    //order this way for nice display
    pub artifact: bool,
    pub enchantment: bool,
    pub planeswalker: bool,
    pub instant: bool,
    pub sorcery: bool,
    pub land: bool,
    pub creature: bool,
}
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Supertypes {
    //order this way for nice display
    pub basic: bool,
    pub legendary: bool,
    pub snow: bool,
}
#[derive(Clone, Copy)]
pub struct Subtypes {
    //needs a manual serialize implementation
    //Would probaboly be needed anyways for JS
    table: BitArr!(for Subtype::VARIANT_COUNT),
}

impl Serialize for Subtypes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer{
            let mut seq = serializer.serialize_seq(None)?;
            for (i,bit) in self.table.iter().enumerate(){
                if *bit{
                    let i:u32=i.try_into().unwrap();
                    let subtype:Subtype=unsafe{
                        std::mem::transmute(i)
                    };
                    seq.serialize_element(&subtype)?;
                }
            }
            seq.end()
        }
}

impl Subtypes {
    pub fn new() -> Self {
        Subtypes {
            table: BitArray::ZERO,
        }
    }
    pub fn add(&mut self, t: Subtype) {
        *self.table.get_mut(t as usize).unwrap() = true;
    }
    pub fn get(&self, t: Subtype) -> bool {
        *self.table.get(t as usize).unwrap()
    }
    pub fn lose_all_subtypes(&mut self) {
        self.table = BitArray::ZERO
    }
}
