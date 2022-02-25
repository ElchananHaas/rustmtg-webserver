use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    hash::Hash,
    num::NonZeroU64,
    ops::DerefMut,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use serde::{ser::SerializeMap, Serialize};
use serde_derive::Serialize;
#[derive(Serialize, Clone)]
pub struct EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64>,
{
    ents: HashMap<K, V>,
    count: usize,
}

const ARENA_CAP: usize = 8;

impl<K, V> EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64>,
{
    pub fn new() -> Self {
        Self {
            ents: HashMap::new(),
            count: 0,
        }
    }
    pub fn view(&self) -> Vec<(K, &V)> {
        let res = Vec::new();
        for (k, v) in self.ents {
            res.push((k, &v));
        }
        res
    }
    pub fn get(&self, id: K) -> Option<&V> {
        self.ents.get(&id)
    }
    pub fn get_mut(&self, id: K) -> Option<&mut V> {
        self.ents.get_mut(&id)
    }

    pub fn remove(&mut self, id: K) -> Option<V> {
        self.ents.remove(&id)
    }
    fn get_newkey(&mut self) -> K {
        self.count += 1;
        let newkey = K::from(self.count);
        newkey
    }
    pub fn insert(&mut self, value: V) -> K {
        let newkey = self.get_newkey();
        self.ents.insert(newkey, value);
        newkey
    }
}
