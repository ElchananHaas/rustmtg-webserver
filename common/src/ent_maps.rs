use std::{collections::HashMap, hash::Hash, num::NonZeroU64};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, JsonSchema, Serialize, Debug)]
pub struct EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64> + JsonSchema,
    V: JsonSchema,
{
    #[serde(flatten)]
    ents: HashMap<K, V>,
    #[serde(skip)]
    count: NonZeroU64,
}

//This hack is to work around https://github.com/serde-rs/serde/issues/1183
impl<'de, K, V> Deserialize<'de> for EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64> + JsonSchema + Deserialize<'de>,
    V: JsonSchema + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let json_val = <serde_json::Value>::deserialize(deserializer)?;
        let mut buf: Vec<u8> = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut json_serial = serde_json::Serializer::new(cursor);
            json_val
                .serialize(&mut json_serial)
                .map_err(serde::de::Error::custom)?;
        }
        //This unsafe is fine becuase serde-json doesn't borrow from its input
        let buf_ref: &[u8] = unsafe { std::mem::transmute(&*buf) };
        let base: HashMap<K, V> =
            serde_json::from_slice(&buf_ref).map_err(serde::de::Error::custom)?;
        Ok(Self {
            ents: base,
            count: NonZeroU64::new(1).unwrap(),
        })
    }
}

impl<K, V> Default for EntMap<K, V>
where
    K: Copy + Hash + Eq + From<NonZeroU64> + JsonSchema,
    V: JsonSchema,
{
    fn default() -> Self {
        Self::new()
    }
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
