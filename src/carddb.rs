use crate::ability::Ability;
use crate::components::{CardName, EntCore, Subtype, Types, PT, ImageUrl};
use crate::cost::Cost;
use crate::game::Color;
use anyhow::{bail, Result};
use hecs::{Entity, EntityBuilder, World};
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fs;
use std::hash::Hash;
//It returns mut cardbuilder due to method chaining
pub type Cardbuildtype = fn(&mut CardBuilder) -> &mut CardBuilder;
pub struct CardDB {
    builders: HashMap<String, Cardbuildtype>,
    scryfall: HashMap<String, Scryfall>,
}

impl fmt::Debug for CardDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CardDB").finish()
    }
}
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ScryfallImageUrls {
    small: Option<String>,
    normal: Option<String>,
    large: Option<String>,
}
#[derive(Deserialize, Debug)]
struct ScryfallEntry {
    object: Option<String>,
    name: String,
    image_uris: Option<ScryfallImageUrls>,
    mana_cost: Option<String>,
    type_line: Option<String>,
}
#[allow(dead_code)]
struct Scryfall {
    object: Option<String>,
    image_uris: Option<ScryfallImageUrls>,
    mana_cost: Option<String>,
    type_line: Option<String>,
}
impl CardDB {
    pub fn new() -> Self {
        let path = "oracle-cards-20211212220409.json";
        let data = fs::read_to_string(path).expect("Couldn't find oracle database file");
        let desered: Vec<ScryfallEntry> = serde_json::from_str(&data).expect("failed to parse!");
        let mut byname = HashMap::new();
        for card in desered {
            byname.insert(
                card.name,
                Scryfall {
                    object: card.object,
                    image_uris: card.image_uris,
                    mana_cost: card.mana_cost,
                    type_line: card.type_line,
                },
            );
        }
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
        CardDB {
            builders: map,
            scryfall: byname,
        }
    }
    pub fn spawn_card(&self, ents: &mut World, card_name: &str, owner: Entity) -> Result<Entity> {
        match self.builders.get(card_name) {
            Some(cardmaker) => {
                let mut builder = CardBuilder::new(card_name.to_owned(), owner);
                let mut url=None;
                if let Some(scryfall_entry)=self.scryfall.get(card_name){
                    if let Some(images)=scryfall_entry.image_uris.as_ref(){
                        if let Some(normal_image)=images.normal.as_ref(){
                            url=Some(normal_image.to_owned());
                        }
                    }
                };
                if let Some(url)=url{
                    builder.add_url(url);
                }
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
    real_card: bool,
    name: String,
    owner: Entity,
}

impl CardBuilder {
    pub fn new(name: String, owner: Entity) -> Self {
        let mut builder = CardBuilder {
            builder: EntityBuilder::new(),
            abilities: Vec::new(),
            types: Types::default(),
            subtypes: HashSet::new(),
            costs: Vec::new(),
            real_card: true,
            name: (&name).clone(),
            owner: owner,
        };
        builder.builder.add(CardName(name));
        builder
    }
    pub fn mana_string(&mut self, coststr: &str) -> &mut Self {
        let mut generic: i32 = 0;
        for letter in coststr.chars() {
            if letter.is_digit(10) {
                generic *= 10;
                //This should be safe bc/ these are hardcoded within the code
                generic += i32::try_from(letter.to_digit(10).unwrap()).unwrap();
            }
            if letter == 'W' {
                self.cost(Cost::Color(Color::White));
            }
            if letter == 'U' {
                self.cost(Cost::Color(Color::Blue));
            }
            if letter == 'B' {
                self.cost(Cost::Color(Color::Black));
            }
            if letter == 'R' {
                self.cost(Cost::Color(Color::Red));
            }
            if letter == 'G' {
                self.cost(Cost::Color(Color::Green));
            }
        }
        for _ in 0..generic {
            self.cost(Cost::Generic);
        }
        self
    }
    pub fn token(&mut self) -> &mut Self {
        self.real_card = false;
        self
    }
    pub fn add_url(&mut self,url:String)-> &mut Self{
        self.builder.add(ImageUrl(url));
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
        self.builder.add(EntCore {
            name: self.name,
            real_card: self.real_card,
            known: HashSet::new(),
            owner: self.owner,
        });
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
