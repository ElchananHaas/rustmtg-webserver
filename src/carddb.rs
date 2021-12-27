use crate::types::Subtype;
use crate::ability::Ability;
use crate::types::Types;
use crate::cost::Cost;
use crate::game::Color;
use anyhow::{bail, Result};
use hecs::{Entity, EntityBuilder, World};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
//It returns mut cardbuilder due to method chaining
pub type Cardbuildtype = fn(&mut CardBuilder) -> &mut CardBuilder;
pub struct CardDB {
    builders: HashMap<String, Cardbuildtype>,
}

impl CardDB {
    pub fn new() -> Self {
        let cards: [(String, Cardbuildtype); 1] = [(
            "Staunch Shieldmate".to_owned(),
            |builder: &mut CardBuilder| {
                builder
                    .creature()
                    .pt(1, 3)
                    .subtype(Subtype::Dwarf)
                    .subtype(Subtype::Soldier)
                    .mana_string("W")
            },
        )];
        let mut map = HashMap::<String, Cardbuildtype>::new();
        for (name, constructor) in cards {
            map.insert(name, constructor);
        }
        CardDB { builders: map }
    }
    pub fn spawn_card(&self, ents: &mut World, card_name: &str) -> Result<Entity> {
        match self.builders.get(card_name) {
            Some(cardmaker) => {
                let mut builder = CardBuilder::new(card_name.to_owned());
                cardmaker(&mut builder);
                let res = ents.spawn(builder.build().build()); //Two build calls
                Ok(res)
                //because builder returns an entitybuilder,
                //and builtentity has lifetime issues
            }
            None => bail!("Card not found"),
        }
    }
}
pub struct CardBuilder {
    builder: EntityBuilder,
    abilities: Vec<Ability>,
    types: Types,
    subtypes: HashSet<Subtype>,
    costs: Vec<Cost>,
    token: bool,
    name: String,
}
impl CardBuilder {
    pub fn new(name:String) -> Self {
        let mut builder=CardBuilder {
            builder: EntityBuilder::new(),
            abilities: Vec::new(),
            types: Types::default(),
            subtypes: HashSet::new(),
            costs: Vec::new(),
            token: false,
            name: (&name).clone()
        };
        builder.builder.add(CardName(name));
        builder
    }
    pub fn mana_string(&mut self,coststr:&str)-> &mut Self{
        let mut generic:i32=0;
        for letter in coststr.chars(){
            if letter.is_digit(10){
                generic*=10;
//This should be safe bc/ these are hardcoded within the code
                generic+=i32::try_from(letter.to_digit(10).unwrap()).unwrap();
            }
            if letter=='W'{
                self.cost(Cost::Color(Color::White));
            }
            if letter=='U'{
                self.cost(Cost::Color(Color::Blue));
            }
            if letter=='B'{
                self.cost(Cost::Color(Color::Black));
            }
            if letter=='R'{
                self.cost(Cost::Color(Color::Red));
            }
            if letter=='G'{
                self.cost(Cost::Color(Color::Green));
            }
        }
        for _ in 0..generic{
            self.cost(Cost::Generic);
        }
        self
    }
    pub fn token(&mut self)-> &mut Self{
        self.token=true;
        self
    }
    pub fn cost(&mut self, cost: Cost) -> &mut Self {
        self.costs.push(cost);
        self
    }
    pub fn pt(&mut self, power: i32, toughness: i32) -> &mut Self {
        self.builder.add(PT {
            power: power,
            toughness: toughness,
        });
        self
    }
    pub fn ability(&mut self, ability: Ability) -> &mut Self {
        self.abilities.push(ability);
        self
    }
    pub fn land(&mut self) -> &mut Self {
        self.types.land = true;
        self
    }
    pub fn creature(&mut self) -> &mut Self {
        self.types.creature = true;
        self
    }
    pub fn enchantment(&mut self) -> &mut Self {
        self.types.enchantment = true;
        self
    }
    pub fn artifact(&mut self) -> &mut Self {
        self.types.artifact = true;
        self
    }
    pub fn planeswalker(&mut self) -> &mut Self {
        self.types.planeswalker = true;
        self
    }
    pub fn instant(&mut self) -> &mut Self {
        self.types.instant = true;
        self
    }
    pub fn sorcery(&mut self) -> &mut Self {
        self.types.sorcery = true;
        self
    }
    pub fn subtype(&mut self, subtype: Subtype) -> &mut Self {
        self.subtypes.insert(subtype);
        self
    }
    pub fn build(mut self) -> EntityBuilder {
        self.builder.add(CardIdentity{name:self.name,token:self.token});
        if !self.abilities.is_empty() {
            self.builder.add(self.abilities);
        };
        if !self.subtypes.is_empty() {
            self.builder.add(self.subtypes);
        };
        self.builder.add(self.types);
        if !self.costs.is_empty() {
            self.builder.add(self.costs);
        };
        self.builder
    }
}
#[derive(Clone, Debug)]
pub struct CardName(String);
#[derive(Clone, Debug)]
pub struct CardIdentity{
    pub name:String,
    pub token:bool,
}
#[derive(Copy, Clone, Debug)]
pub struct PT {
    pub power: i32,
    pub toughness: i32,
}

