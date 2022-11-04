use std::{collections::HashMap, hash::Hash, num::NonZeroU64};

use schemars::JsonSchema;
use serde::Serialize;

use crate::{card_entities::CardEnt, entities::CardId, spellabil::KeywordAbility};
#[derive(Clone, JsonSchema, Serialize, Debug)]
pub struct EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64> + JsonSchema,
    V: JsonSchema,
{
    ents: HashMap<K, V>,
    #[serde(skip)]
    count: NonZeroU64,
}

impl<K, V> EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64> + JsonSchema,
    V: JsonSchema,
{
    pub fn new() -> Self {
        Self {
            ents: HashMap::new(),
            count: NonZeroU64::new(1).unwrap(),
        }
    }
    pub fn view(&self) -> Vec<(K, &V)> {
        let mut res = Vec::new();
        for (k, v) in &self.ents {
            res.push((*k, v));
        }
        res
    }
    pub fn get(&self, id: K) -> Option<&V> {
        self.ents.get(&id)
    }
    pub fn get_mut(&mut self, id: K) -> Option<&mut V> {
        self.ents.get_mut(&id)
    }
    pub fn is(&self, id: K, f: impl FnOnce(&V) -> bool) -> bool {
        match self.ents.get(&id) {
            None => false,
            Some(ent) => f(ent),
        }
    }
    pub fn peek_count(&self) -> NonZeroU64 {
        self.count
    }
    pub fn remove(&mut self, id: K) -> Option<V> {
        self.ents.remove(&id)
    }
    fn get_newkey(&mut self) -> K {
        let newkey = K::from(self.count);
        let val = self.count.get() + 1;
        self.count = NonZeroU64::new(val).unwrap();
        newkey
    }
    pub fn insert(&mut self, value: V) -> (K, &mut V) {
        let newkey = self.get_newkey();
        self.ents.insert(newkey, value);
        (newkey, self.ents.get_mut(&newkey).unwrap())
    }
    pub fn skip_count(&mut self, n: u64) {
        let val = self.count.get() + n;
        self.count = NonZeroU64::new(val).unwrap();
    }
}
impl EntMap<CardId, CardEnt> {
    pub fn has_keyword(&self, ent: CardId, keyword: KeywordAbility) -> bool {
        self.is(ent, |card| card.has_keyword(keyword))
    }
}
