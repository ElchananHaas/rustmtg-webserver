//This file stores varius components that entities may
use hecs::Entity;
use serde_derive::Serialize;
use std::{collections::HashSet};

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Damage(pub i32);

//Utility structure for figuring out if a creature can tap
//Added the turn it ETBs or changes control
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct SummoningSickness();
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Tapped();
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct DealtCombatDamage();
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Attacking(pub Entity);
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Blocked(pub Vec<Entity>);

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Blocking(pub Vec<Entity>);
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct CardName(pub String);
#[derive(Clone, Debug, Serialize)]
pub struct EntCore {
    pub owner: Entity,
    pub name: String,
    pub real_card: bool,
    pub known: HashSet<Entity>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ImageUrl(pub String);
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PT {
    pub power: i32,
    pub toughness: i32,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Controller(pub Entity);

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Types {
    pub land: bool,
    pub creature: bool,
    pub artifact: bool,
    pub enchantment: bool,
    pub planeswalker: bool,
    pub instant: bool,
    pub sorcery: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize)]
#[allow(dead_code)] //allow dead code to reduce warnings noise on each variant
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
}
