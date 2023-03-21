use nom::error::VerboseError;
use nom::IResult;
use schemars::JsonSchema;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::hash::Hash;
use std::str::FromStr;
use strum_macros::EnumString;
use texttoken::Tokens;

use crate::hashset_obj::HashSetObj;

pub trait ParseType {
    fn parse<'a>(x: &'a Tokens) -> IResult<&'a Tokens, Self, VerboseError<&'a Tokens>>
    where
        Self: Sized,
        Self: FromStr,
    {
        if x.len() == 0 {
            return Err(nom::Err::Error(VerboseError {
                errors: vec![(
                    x,
                    nom::error::VerboseErrorKind::Context("Empty string when parsing type"),
                )],
            }));
        }
        match Self::from_str(&x[0]) {
            Ok(val) => Ok((&x[1..], val)),
            Err(_) => Err(nom::Err::Error(VerboseError {
                errors: vec![(
                    x,
                    nom::error::VerboseErrorKind::Context("Failed to parse type"),
                )],
            })),
        }
    }
    fn parse_set<'a>(
        x: &'a Tokens,
    ) -> IResult<&'a Tokens, HashSetObj<Self>, VerboseError<&'a Tokens>>
    where
        Self: Sized,
        Self: FromStr,
        Self: Hash,
        Self: Eq,
    {
        let (x, types) = nom::multi::many0(Self::parse)(x)?;
        return Ok((x, types.into_iter().collect()));
    }
}
impl ParseType for Type {}
impl ParseType for Supertype {}
impl ParseType for Subtype {}
pub type Types = HashSetObj<Type>;
pub type Subtypes = HashSetObj<Subtype>;
pub type Supertypes = HashSetObj<Supertype>;
impl HashSetObj<Type> {
    pub fn is_creature(&self) -> bool {
        self.contains(&Type::Creature)
    }
    pub fn is_land(&self) -> bool {
        self.contains(&Type::Land)
    }
    pub fn is_instant(&self) -> bool {
        self.contains(&Type::Instant)
    }
    pub fn is_sorcery(&self) -> bool {
        self.contains(&Type::Sorcery)
    }
    pub fn is_artifact(&self) -> bool {
        self.contains(&Type::Artifact)
    }
    pub fn is_enchantment(&self) -> bool {
        self.contains(&Type::Enchantment)
    }
    pub fn is_planeswalker(&self) -> bool {
        self.contains(&Type::Planeswalker)
    }
}
#[derive(
    Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, JsonSchema, EnumString,
)]
#[strum(serialize_all = "lowercase")]
pub enum Type {
    Artifact,
    Enchantment,
    Planeswalker,
    Land,
    Creature,
    Instant,
    Sorcery,
}
#[derive(
    Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, JsonSchema, EnumString,
)]
#[strum(serialize_all = "lowercase")]
pub enum Supertype {
    Basic,
    World,
    Legendary,
    Snow,
}
#[derive(
    Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, JsonSchema, EnumString,
)]
#[strum(serialize_all = "lowercase")]
pub enum Subtype {
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
    Forest,
}
