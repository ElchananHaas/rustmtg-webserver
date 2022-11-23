use nom::IResult;
use paste::paste;
use schemars::JsonSchema;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::str::FromStr;
use strum_macros::EnumString;
macro_rules! enumset{
    ($name:ident, $($e:ident),*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, JsonSchema, EnumString)]
        #[strum(serialize_all = "lowercase")]
        #[allow(dead_code)] //allow dead code to reduce warnings noise on each variant
        #[repr(u32)]
        pub enum $name{
            $($e,)*
        }
        #[allow(dead_code)]
        impl $name{
            pub fn parse(x:&[String])->IResult<&[String], Self, ()>{
                if x.len()==0 {
                    return Err(nom::Err::Error(()));
                }
                match $name::from_str(&x[0]){
                    Ok(val)=>Ok((&x[1..],val)),
                    Err(_)=>Err(nom::Err::Error(()))
                }
            }
        }
        paste!{
            #[derive(Default)]
            #[derive(Clone, Copy, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
            pub struct [<$name s>]{
                $(
                    pub [<$e:lower>]:bool,
                )*
            }
            impl std::fmt::Debug for [<$name s>]{
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                    write!(f, "[")?;
                    $(
                        if(self.[<$e:lower>]){
                            $name::$e.fmt(f)?;
                           write!(f, ",")?;
                        }

                    )*
                    write!(f, "]")?;
                    Ok(())

                }
            }
            #[allow(dead_code)]
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
                pub fn parse(x:&[String])->IResult<&[String], Self, ()>{
                    let mut res=Self::new();
                    let typed:IResult<&[String],Vec<$name>,()>=nom::multi::many0($name::parse)(x);
                    match typed{
                        Ok((rest,types))=>{
                            for t in types{
                                res.add(t);
                            }
                            return Ok((rest,res))
                        },
                        Err(e)=>{
                            return Err(e)
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
