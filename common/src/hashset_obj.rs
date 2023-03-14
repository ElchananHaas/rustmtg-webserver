//This is a hashset with modified serialization to serialize as a javascript object, not an array.
//This will make the front end far simpler and reduce bugs there
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct HashSetObj<T>
where
    T: Hash + Eq,
{
    base: HashMap<T, ()>,
}

impl<T> HashSetObj<T>
where
    T: Hash + Eq,
{
    pub fn insert(&mut self, value: T) -> bool {
        self.base.insert(value, ()).is_some()
    }
    pub fn add(&mut self, value: T) -> bool {
        self.insert(value)
    }
    pub fn remove(&mut self, value: &T) -> bool {
        self.base.remove(value).is_some()
    }
    pub fn new() -> Self {
        HashSetObj {
            base: HashMap::new(),
        }
    }
    pub fn contains(&self, value: &T) -> bool {
        self.base.contains_key(value)
    }
    pub fn get(&self, value: &T) -> bool {
        self.contains(value)
    }
    pub fn len(&self) -> usize {
        self.base.len()
    }
    pub fn is_subset(&self, other: &HashSetObj<T>) -> bool {
        for key in other.iter() {
            if !self.contains(key) {
                return false;
            }
        }
        true
    }
}
impl<T> Default for HashSetObj<T>
where
    T: Hash + Eq,
{
    fn default() -> Self {
        Self {
            base: Default::default(),
        }
    }
}
//This hack is to work around https://github.com/serde-rs/serde/issues/1183
impl<'de,T> Deserialize<'de> for HashSetObj<T>
where
    T: Hash + Eq + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let json_val=<serde_json::Value>::deserialize(deserializer)?;
        let mut buf:Vec<u8>=Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut json_serial = serde_json::Serializer::new(cursor);
            json_val.serialize(&mut json_serial).map_err(serde::de::Error::custom)?;
        }
        //This unsafe is fine becuase serde-json doesn't borrow from its input
        let buf_ref:&[u8]=unsafe{std::mem::transmute(&*buf)};
        let base: HashMap<T,()> = serde_json::from_slice(&buf_ref).map_err(serde::de::Error::custom)?;
        Ok(Self { base})
        

    }
}
impl<'a, T> HashSetObj<T>
where
    T: Hash + Eq,
{
    pub fn iter(&'a self) -> std::collections::hash_map::Keys<'a, T, ()> {
        self.into_iter()
    }
}

impl<T> FromIterator<T> for HashSetObj<T>
where
    T: Hash + Eq,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = HashMap::new();

        for i in iter {
            c.insert(i, ());
        }
        Self { base: c }
    }
}
impl<T> IntoIterator for HashSetObj<T>
where
    T: Hash + Eq,
{
    type Item = T;
    type IntoIter = <std::collections::hash_map::IntoKeys<T, ()> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.base.into_keys()
    }
}

impl<'a, T> IntoIterator for &'a HashSetObj<T>
where
    T: Hash + Eq,
{
    type Item = &'a T;
    type IntoIter = <std::collections::hash_map::Keys<'a, T, ()> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.base.keys()
    }
}
