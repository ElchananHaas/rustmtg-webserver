use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    hash::Hash,
    num::NonZeroU64,
    sync::{Mutex, Arc, atomic::{AtomicU64, Ordering}}, ops::DerefMut,
};
pub struct EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64>,
{
    ents: HashMap<K, V>,
    appends: Vec<(K, Box<V>)>,
    count: AtomicU64,
}

impl<K,V> Clone for EntMap<K,V> where
K: Copy + Hash + Eq + From<NonZeroU64>,V:Clone{
    fn clone(&self) -> Self { 
        Self{
            ents:self.ents.clone(),
            appends:self.appends.clone(),
            count: AtomicU64::new(self.count.load(Ordering::Acquire))
        }
     }
}
const ARENA_CAP: usize = 8;

impl<K, V> EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64>,
{
    pub fn new() -> Self {
        Self {
            ents: HashMap::new(),
            appends: Vec::new(), //By default hold space for 1 element,
            //which is probably what I want
            count: AtomicU64::new(1),
        }
    }
    pub fn view(&self) -> Vec<(K, &V)> {
        let res = Vec::new();
        for (k, v) in self.ents {
            res.push((k, &v));
        }
        for (k, v) in self.appends {
            res.push((k, &*v));
        }
        res
    }
    pub fn get(&self, id: K) -> Option<&V> {
        match self.ents.get(&id) {
            Some(v) => Some(v),
            None => self.appends.iter().find_map(|(key, val)| {
                let interior = &**val;
                if *key == id {
                    Some(interior)
                } else {
                    None
                }
            }),
        }
    }
    pub fn get_mut(&self, id: K) -> Option<&mut V> {
        match self.ents.get_mut(&id) {
            Some(v) => Some(v),
            None => self.appends.iter_mut().find_map(|(key, val)| {
                let mut interior = val.deref_mut();
                if *key == id {
                    Some(interior)
                } else {
                    None
                }
            }),
        }
    }
    //Flush all inserts. Call before every mutable function
    pub fn flush_inserts(&mut self) {
        for (key, val) in self.appends.drain(..) {
            self.ents.insert(key, *val);
        }
    }
    pub fn remove(&mut self, id: K) -> Option<V> {
        self.flush_inserts();
        self.ents.remove(&id)
    }
    fn get_newkey(&self) -> K {
        let count = self.count.load(std::sync::atomic::Ordering::Acquire);
        self.count.fetch_add(1, std::sync::atomic::Ordering::Release);
        let count=NonZeroU64::try_from(count).unwrap();
        let newkey = K::from(count);
        newkey
    }
    pub fn insert(&mut self, value: V) -> K {
        self.flush_inserts();
        let newkey = self.get_newkey();
        self.ents.insert(newkey, value);
        newkey
    }
    //This function inserts a key usng interior mutability/
    //It is less efficient than insert, so use only when necessary
    pub fn insert_immutable(&self, value: V) -> K {
        let newkey: K = self.get_newkey();
        self.appends.push((newkey, Box::new(value)));
        newkey
    }
}
