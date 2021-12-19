use hecs::{World,Entity,EntityBuilder};
use anyhow::{Result,bail};
use crate::subtypes::Subtype;
use std::collections::{HashSet,HashMap};
use std::fmt;
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
    costs:Vec<Cost>,
}
impl CardBuilder{
    pub fn new()->Self{
        CardBuilder{
            builder:EntityBuilder::new(),
            abilities:Vec::new(),
            types:Types::default(),
            subtypes:HashSet::new(),
            costs:Vec::new(),
        }
    }
    pub fn cost(&mut self,cost:Cost)->&mut Self{
        self.costs.push(cost);
        self
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
pub struct TriggeredAbility{

}
#[derive(Clone)]
pub struct ActivatedAbility{
    mana_ability:bool,
    effect:Cardbuildtype
}
impl fmt::Debug for ActivatedAbility{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActivatedAbility")
         .field("mana_ability", &self.mana_ability)
         .finish()
    }
}
#[derive(Clone,Debug)]
pub struct StaticAbility{

}
#[derive(Clone,Debug)]
pub enum Ability{
    Activated(ActivatedAbility),
    Triggered(TriggeredAbility),
    Static(StaticAbility),
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
pub enum Cost{
    Generic(i32),
    White(i32),
    Blue(i32),
    Black(i32),
    Red(i32),
    Green(i32),
    Selftap,
}
impl Cost{
    //Takes in the game, the player paying the cost and a vector of 
    //objects used to pay the cost. Returns a vector of references
    //to the entities used to pay the costs, or an error if it could not
    //be paid. 
    //Also includes the source for prevention effects
    pub fn pay(&self,game:&mut Game,source:Entity,player:Entity,payment:Vec<Entity>)->Result<Vec<Entity>>{
        match self{
            Generic(x)=>
        }
    }
}