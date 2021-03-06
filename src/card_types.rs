use crate::carddb::trim_spaces;
use nom;
use nom::bytes::complete::tag;
use nom::error::{Error, ParseError};
use nom::Err;
use nom::IResult;
use paste::paste;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_derive::Serialize;
use std::convert::AsRef;
use strum_macros::AsRefStr;
macro_rules! enumset{
    ($name:ident, $($e:ident),*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize)]
        #[derive(AsRefStr)]
        #[allow(dead_code)] //allow dead code to reduce warnings noise on each variant
        #[repr(u32)]
        pub enum $name{
            $($e,)*
        }
        impl $name{
            pub fn parse(x:&str)->IResult<&str, Self>{
                $(
                    let parse:IResult<&str,&str>=tag($name::$e.as_ref())(x);
                    if let Ok((rest,_))=parse{
                        return Ok((rest,$name::$e));
                    }
                )*
                return Err(Err::Error(Error::from_error_kind(x, nom::error::ErrorKind::Alt)));
            }
        }
        paste!{
            #[derive(Default)]
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            pub struct [<$name s>]{
                $(
                    pub [<$e:lower>]:bool,
                )*
            }
            impl Serialize for [<$name s>]{
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut seq = serializer.serialize_seq(None)?;
                    $(
                        if(self.[<$e:lower>]){
                            seq.serialize_element(&$name::$e)?;
                        }
                    )*
                    seq.end()
                }
            }
            impl [<$name s>]{
                pub fn new()->Self{
                    Self::default()
                }
                pub fn get(&self,x:$name)->bool{
                    match x{
                        $(
                            $name::$e=>self.[<$e:lower>],
                        )*
                    }
                }
                pub fn add(&mut self,x:$name){
                    match x{
                        $(
                            $name::$e=>{self.[<$e:lower>]=true},
                        )*
                    }
                }
                pub fn remove(&mut self,x:$name){
                    match x{
                        $(
                            $name::$e=>{self.[<$e:lower>]=false},
                        )*
                    }
                }
                pub fn remove_all(&mut self){
                    *self=Default::default();
                }
                //TODO handle spaces-or lex, then parse
                pub fn parse(mut x:&str)->IResult<&str, Self>{
                    let mut res=Self::new();
                    loop{
                        (x,_)=trim_spaces(x)?;
                        if let Ok((rest,t))=$name::parse(x){
                            x=rest;
                            res.add(t);
                        }else{
                            return Ok((x,res));
                        }
                    }
                }
            }
        }
    };
}
enumset!(
    Type,
    Artifact,
    Enchantment,
    Planeswalker,
    Land,
    Creature,
    Instant,
    Sorcery
);
enumset!(Supertype, Basic, World, Legendary, Snow);
enumset!(
    Subtype,
    Advisor,
    Aetherborn,
    Ally,
    Angel,
    Antelope,
    Ape,
    Archer,
    Archon,
    Army,
    Artificer,
    Assassin,
    AssemblyWorker,
    Atog,
    Aurochs,
    Avatar,
    Azra,
    Badger,
    Barbarian,
    Bard,
    Basilisk,
    Bat,
    Bear,
    Beast,
    Beeble,
    Beholder,
    Berserker,
    Bird,
    Blinkmoth,
    Boar,
    Bringer,
    Brushwagg,
    Camarid,
    Camel,
    Caribou,
    Carrier,
    Cat,
    Centaur,
    Cephalid,
    Chimera,
    Citizen,
    Cleric,
    Cockatrice,
    Construct,
    Coward,
    Crab,
    Crocodile,
    Cyclops,
    Dauthi,
    Demigod,
    Demon,
    Deserter,
    Devil,
    Dinosaur,
    Djinn,
    Dog,
    Dragon,
    Drake,
    Dreadnought,
    Drone,
    Druid,
    Dryad,
    Dwarf,
    Efreet,
    Egg,
    Elder,
    Eldrazi,
    Elemental,
    Elephant,
    Elf,
    Elk,
    Eye,
    Faerie,
    Ferret,
    Fish,
    Flagbearer,
    Fox,
    Fractal,
    Frog,
    Fungus,
    Gargoyle,
    Germ,
    Giant,
    Gnoll,
    Gnome,
    Goat,
    Goblin,
    God,
    Golem,
    Gorgon,
    Graveborn,
    Gremlin,
    Griffin,
    Hag,
    Halfling,
    Hamster,
    Harpy,
    Hellion,
    Hippo,
    Hippogriff,
    Homarid,
    Homunculus,
    Horror,
    Horse,
    Human,
    Hydra,
    Hyena,
    Illusion,
    Imp,
    Incarnation,
    Inkling,
    Insect,
    Jackal,
    Jellyfish,
    Juggernaut,
    Kavu,
    Kirin,
    Kithkin,
    Knight,
    Kobold,
    Kor,
    Kraken,
    Lamia,
    Lammasu,
    Leech,
    Leviathan,
    Lhurgoyf,
    Licid,
    Lizard,
    Manticore,
    Masticore,
    Mercenary,
    Merfolk,
    Metathran,
    Minion,
    Minotaur,
    Mole,
    Monger,
    Mongoose,
    Monk,
    Monkey,
    Moonfolk,
    Mouse,
    Mutant,
    Myr,
    Mystic,
    Naga,
    Nautilus,
    Nephilim,
    Nightmare,
    Nightstalker,
    Ninja,
    Noble,
    Noggle,
    Nomad,
    Nymph,
    Octopus,
    Ogre,
    Ooze,
    Orb,
    Orc,
    Orgg,
    Otter,
    Ouphe,
    Ox,
    Oyster,
    Pangolin,
    Peasant,
    Pegasus,
    Pentavite,
    Pest,
    Phelddagrif,
    Phoenix,
    Phyrexian,
    Pilot,
    Pincher,
    Pirate,
    Plant,
    Praetor,
    Prism,
    Processor,
    Rabbit,
    Ranger,
    Rat,
    Rebel,
    Reflection,
    Rhino,
    Rigger,
    Rogue,
    Sable,
    Salamander,
    Samurai,
    Sand,
    Saproling,
    Satyr,
    Scarecrow,
    Scion,
    Scorpion,
    Scout,
    Sculpture,
    Serf,
    Serpent,
    Servo,
    Shade,
    Shaman,
    Shapeshifter,
    Shark,
    Sheep,
    Siren,
    Skeleton,
    Slith,
    Sliver,
    Slug,
    Snake,
    Soldier,
    Soltari,
    Spawn,
    Specter,
    Spellshaper,
    Sphinx,
    Spider,
    Spike,
    Spirit,
    Splinter,
    Sponge,
    Squid,
    Squirrel,
    Starfish,
    Surrakar,
    Survivor,
    Tentacle,
    Tetravite,
    Thalakos,
    Thopter,
    Thrull,
    Tiefling,
    Treefolk,
    Trilobite,
    Triskelavite,
    Troll,
    Turtle,
    Unicorn,
    Vampire,
    Vedalken,
    Viashino,
    Volver,
    Wall,
    Warlock,
    Warrior,
    Weird,
    Werewolf,
    Whale,
    Wizard,
    Wolf,
    Wolverine,
    Wombat,
    Worm,
    Wraith,
    Wurm,
    Yeti,
    Zombie,
    Zubera,
    EndCreatureMarker,
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest
);
