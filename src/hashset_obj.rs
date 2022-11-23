//This is a hashset with modified serialization to serialize as a javascript object, not an array.
//This will make the front end far simpler and reduce bugs there
use std::collections::HashMap;
use schemars::JsonSchema;
use serde::{self, Serializer, Serialize, Deserialize, Deserializer};
use std::hash::Hash;
#[derive(Clone,Debug,Default)]
pub struct HashSetObj<T>
where T:Hash+Eq{
    base:HashMap<T,()>
}

impl<T> HashSetObj<T>
where T:Hash+Eq{
    pub fn insert(&mut self, value: T) -> bool {
        self.base.insert(value,()).is_some()
    }
    pub fn remove(&mut self,value: &T) -> bool {
        self.base.remove(value).is_some()
    }
    pub fn new() -> Self {
        HashSetObj { base: HashMap::new() }
    }
    pub fn contains(&self,value: &T) -> bool {
        self.base.contains_key(value)
    }
    pub fn len(&self) -> usize{
        self.base.len()
    }
    pub fn is_subset(&self,other:&HashSetObj<T>)->bool{
        for key in other.iter(){
            if !self.contains(key){
                return false;
            }
        }
        true
    }
}

impl<'a,T> HashSetObj<T>
where T:Hash+Eq{
    pub fn iter(&'a self) -> std::collections::hash_map::Keys<'a, T, ()> {
        self.into_iter()
    }
}

impl<T> FromIterator<T> for HashSetObj<T>
where T:Hash+Eq{
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut c = HashMap::new();

        for i in iter {
            c.insert(i, ());
        }
        Self{
            base:c
        }
    }
}
impl<T> IntoIterator for HashSetObj<T>
where T:Hash+Eq{
    type Item = T;
    type IntoIter = <std::collections::hash_map::IntoKeys<T, ()> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.base.into_keys()
    }
}

impl<'a,T> IntoIterator for &'a HashSetObj<T>
where T:Hash+Eq{
    type Item = &'a T;
    type IntoIter = <std::collections::hash_map::Keys<'a , T, ()> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.base.keys()
    }
}
impl<T: Serialize + Hash + Eq> Serialize for HashSetObj<T>{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<<S as Serializer>::Ok, <S as serde::Serializer>::Error> 
    where S: serde::Serializer { 
        self.base.serialize(serializer)
     }
}

impl<'de, T: Deserialize<'de> + Hash + Eq> Deserialize<'de> for HashSetObj<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Ok(HashSetObj{
            base: <HashMap<T,()> as Deserialize>::deserialize(deserializer)?
        })
    }
}
impl<T> schemars::JsonSchema for HashSetObj<T>
where T:Hash+Eq {
    fn schema_name() -> std::string::String {
        <HashMap<T,()> as JsonSchema>::schema_name()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <HashMap<T,()> as JsonSchema>::json_schema(gen)
    }
}