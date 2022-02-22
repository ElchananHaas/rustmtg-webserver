use std::{collections::HashMap, hash::Hash, cell::{RefCell, Cell}, num::NonZeroU64};


pub struct AppendableMap<K,V> where K:Copy+Hash+Eq+From<NonZeroU64>{
    ents:HashMap<K,V>,
    appends:Vec<(K,Box<V>)>,
    count:Cell<NonZeroU64>,
}
const ARENA_CAP:usize=8;

impl<K,V> AppendableMap<K,V> where K:Copy+Hash+Eq+From<NonZeroU64>{
    pub fn new()->Self{
        Self{
            ents:HashMap::new(),
            appends:Vec::new(),//By default hold space for 1 element,
            //which is probably what I want
            count:Cell::new(NonZeroU64::new(1).unwrap()),
        }
    }

    pub fn get(&self,id:K)->Option<&V>{
        match self.ents.get(&id){
            Some(v)=>Some(v),
            None=>{
                self.appends.iter().find_map(|(key,val)|
                    {
                        let interior=&**val;
                        if *key==id{
                            Some(interior)
                        }else{
                            None
                        }
                    }
                )
            }
        }
    }
    //Flush all inserts. Call before every mutable function
    pub fn flush_inserts(&mut self){
        for (key,val) in self.appends.drain(..){
            self.ents.insert(key, *val);
        }
    }
    pub fn remove(&mut self,id:K)->Option<V>{
        self.flush_inserts();
        self.ents.remove(&id)
    }
    fn get_newkey(&self)->K{
        let count=self.count.get();
        self.count.set(NonZeroU64::new(count.get()+1).unwrap());
        let newkey=K::from(count);
        newkey
    }
    pub fn insert(&mut self,value:V)->K{
        self.flush_inserts();
        let newkey=self.get_newkey();
        self.ents.insert(newkey, value);
        newkey
    }
    //This function inserts a key usng interior mutability/
    //It is less efficient than insert, so use only when necessary
    pub fn insert_immutable(&self,value:V)->K{
        let newkey: K=self.get_newkey();
        self.appends.push((newkey,Box::new(value)));
        newkey
    }
}