
#[derive(Clone,Copy,PartialEq,Eq,Debug,Hash)]
#[repr(C)] //Will be needed for shenanigans to let me
//iterate over creature subtypes
#[allow(dead_code)]
pub enum Subtype{
Advisor, Aetherborn, Ally, Angel, Antelope, Ape, Archer, Archon, Army,
Artificer, Assassin, AssemblyWorker, Atog, Aurochs, Avatar, Azra, 
Badger, Barbarian, Bard, Basilisk, Bat, Bear, Beast, Beeble, Beholder, Berserker,
Bird, Blinkmoth, Boar, Bringer, Brushwagg, Camarid, Camel, Caribou, Carrier, Cat,
Centaur, Cephalid, Chimera, Citizen, Cleric, Cockatrice, Construct, Coward,
Crab, Crocodile, Cyclops, Dauthi, Demigod, Demon, Deserter, Devil, Dinosaur,
Djinn, Dog, Dragon, Drake, Dreadnought, Drone, Druid, Dryad, Dwarf, 
Efreet, Egg, Elder, Eldrazi, Elemental, Elephant, Elf, Elk, Eye, Faerie, 
Ferret, Fish, Flagbearer, Fox, Fractal, Frog, Fungus, Gargoyle, Germ, Giant,
Gnoll, Gnome, Goat, Goblin, God, Golem, Gorgon, Graveborn, Gremlin, Griffin, 
Hag, Halfling, Hamster, Harpy, Hellion, Hippo, Hippogriff, Homarid, Homunculus, 
Horror, Horse, Human, Hydra, Hyena, Illusion, Imp, Incarnation, Inkling, Insect, 
Jackal, Jellyfish, Juggernaut, Kavu, Kirin, Kithkin, Knight, Kobold, Kor, Kraken, 
Lamia, Lammasu, Leech, Leviathan, Lhurgoyf, Licid, Lizard, Manticore, Masticore, 
Mercenary, Merfolk, Metathran, Minion, Minotaur, Mole, Monger, Mongoose, Monk, 
Monkey, Moonfolk, Mouse, Mutant, Myr, Mystic, Naga, Nautilus, Nephilim, Nightmare, 
Nightstalker, Ninja, Noble, Noggle, Nomad, Nymph, Octopus, Ogre, Ooze, Orb, Orc, 
Orgg, Otter, Ouphe, Ox, Oyster, Pangolin, Peasant, Pegasus, Pentavite, Pest, Phelddagrif, 
Phoenix, Phyrexian, Pilot, Pincher, Pirate, Plant, Praetor, Prism, Processor, Rabbit, 
Ranger, Rat, Rebel, Reflection, Rhino, Rigger, Rogue, Sable, Salamander, Samurai, Sand, 
Saproling, Satyr, Scarecrow, Scion, Scorpion, Scout, Sculpture, Serf, Serpent, 
Servo, Shade, Shaman, Shapeshifter, Shark, Sheep, Siren, Skeleton, Slith, Sliver, 
Slug, Snake, Soldier, Soltari, Spawn, Specter, Spellshaper, Sphinx, Spider, Spike, 
Spirit, Splinter, Sponge, Squid, Squirrel, Starfish, Surrakar, Survivor, Tentacle, 
Tetravite, Thalakos, Thopter, Thrull, Tiefling, Treefolk, Trilobite, Triskelavite, 
Troll, Turtle, Unicorn, Vampire, Vedalken, Viashino, Volver, Wall, Warlock, Warrior, 
Weird, Werewolf, Whale, Wizard, Wolf, Wolverine, Wombat, Worm, Wraith, Wurm, Yeti, Zombie, Zubera,
EndCreatureMarker,
}
