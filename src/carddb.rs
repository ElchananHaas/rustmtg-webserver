use hecs::{World,Entity,EntityBuilder};
use anyhow::{Result,bail};
use crate::subtypes::Subtype;
use std::collections::{HashSet,HashMap};

//It returns mut cardbuilder due to method chaining
type Cardbuildtype=fn(&mut CardBuilder)->&mut CardBuilder;
pub struct CardDB{
    builders:HashMap<String,Cardbuildtype>
}

impl CardDB{
    pub fn new()->Self{
        let cards:[(String,Cardbuildtype);1]=[
                ("Staunch Shieldmate".to_owned(), |builder: &mut CardBuilder| 
builder.creature().pt(1,3).subtype(Subtype::Dwarf).subtype(Subtype::Soldier)  )
            ];
        let mut map=HashMap::<String,Cardbuildtype>::new();
        for (name,constructor) in cards{
            map.insert(name,constructor);
        }
        CardDB{
            builders:map
        }
    }
    pub fn spawn_card(&self,ents:&mut World,card_name:&str)->Result<Entity>{
        match self.builders.get(card_name){
            Some(cardmaker)=>{
                let mut builder=CardBuilder::new();
                builder.name(card_name.to_owned());
                cardmaker(&mut builder);
                let res=ents.spawn(builder.build().build());//Two build calls
                Ok(res)
                //because builder returns an entitybuilder, 
                //and builtentity has lifetime issues
            }
            None=> bail!("Card not found")
        }
    }
}
pub struct CardBuilder{
    builder:EntityBuilder,
    abilities:Vec<Ability>,
    types:Types,
    subtypes:HashSet<Subtype>,
}
impl CardBuilder{
    pub fn new()->Self{
        CardBuilder{
            builder:EntityBuilder::new(),
            abilities:Vec::new(),
            types:Types::default(),
            subtypes:HashSet::new(),
        }
    }
    pub fn pt(&mut self,power:i32,toughness:i32)->&mut Self{
        self.builder.add(PT{power:power,toughness:toughness});
        self
    }
    pub fn name(&mut self,name:String)->&mut Self{
        self.builder.add(CardName(name));
        self
    }
    pub fn ability(&mut self,ability:Ability)->&mut Self{
        self.abilities.push(ability);
        self
    }
    pub fn land(&mut self)->&mut Self{
        self.types.land=true;
        self
    }
    pub fn creature(&mut self)->&mut Self{
        self.types.creature=true;
        self
    }
    pub fn enchantment(&mut self)->&mut Self{
        self.types.enchantment=true;
        self
    }
    pub fn artifact(&mut self)->&mut Self{
        self.types.artifact=true;
        self
    }
    pub fn planeswalker(&mut self)->&mut Self{
        self.types.planeswalker=true;
        self
    }
    pub fn instant(&mut self)->&mut Self{
        self.types.instant=true;
        self
    }
    pub fn sorcery(&mut self)->&mut Self{
        self.types.sorcery=true;
        self
    }
    pub fn subtype(&mut self,subtype:Subtype)->&mut Self{
        self.subtypes.insert(subtype);
        self
    }
    pub fn build(mut self)->EntityBuilder{
        if !self.abilities.is_empty() {self.builder.add(self.abilities);};
        if !self.subtypes.is_empty() {self.builder.add(self.subtypes);};
        self.builder
    }
}
#[derive(Clone,Debug)]
pub struct CardName(String);
#[derive(Copy,Clone,Debug)]
pub struct PT{
    power:i32,
    toughness:i32
}
#[derive(Clone,Debug)]
pub struct Ability{
}

#[derive(Clone,Copy,Debug)]
pub enum KeywordAbility{
    FirstStrike,
    Flying,
    Haste,
}
#[derive(Clone,Copy,Debug,Default)]
pub struct Types{
    land:bool,
    creature:bool,
    artifact:bool,
    enchantment:bool,
    planeswalker:bool,
    instant:bool,
    sorcery:bool,
}