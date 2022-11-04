/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type ClientMessage =
  | {
      GameState: GameState;
    }
  | {
      AskUser: Ask;
    };
export type PlayerId = number;
export type CardId = number;
export type Ability =
  | {
      Activated: ActivatedAbility;
    }
  | {
      Triggered: TriggeredAbility;
    }
  | {
      Static: StaticAbility;
    };
export type Cost =
  | "Selftap"
  | {
      Mana: ManaCostSymbol;
    };
export type ManaCostSymbol = "White" | "Blue" | "Black" | "Red" | "Green" | "Colorless" | "Generic";
export type Clause =
  | "DrawCard"
  | {
      AddMana: ManaCostSymbol[];
    };
export type KeywordAbility =
  | "FirstStrike"
  | "Haste"
  | "Vigilance"
  | "DoubleStrike"
  | "Flying"
  | "Prowess"
  | "Lifelink"
  | "Trample"
  | "Reach";
export type TargetId = number;
export type EntType = "RealCard" | "TokenCard" | "ActivatedAbility" | "TriggeredAbility";
export type Color = "White" | "Blue" | "Black" | "Red" | "Green" | "Colorless";
export type GameOutcome =
  | ("Ongoing" | "Tie")
  | {
      Winner: PlayerId;
    };
export type Phase = "Begin" | "FirstMain" | "Combat" | "SecondMain" | "Ending";
export type ManaId = number;
export type Subphase =
  | "Untap"
  | "Upkeep"
  | "Draw"
  | "BeginCombat"
  | "Attackers"
  | "Blockers"
  | "FirstStrikeDamage"
  | "Damage"
  | "EndCombat"
  | "EndStep"
  | "Cleanup";
export type Ask =
  | {
      Attackers: AskPairABFor_TargetId;
    }
  | {
      Blockers: AskPairABFor_CardId;
    }
  | {
      DiscardToHandSize: AskSelectNFor_CardId;
    }
  | {
      Action: AskSelectNFor_Action;
    };
export type Action =
  | {
      Cast: CastingOption;
    }
  | {
      PlayLand: CardId;
    }
  | {
      ActivateAbility: {
        index: number;
        source: CardId;
        [k: string]: unknown;
      };
    };
export type ActionFilter = "None";
export type Zone = "Hand" | "Library" | "Exile" | "Battlefield" | "Graveyard" | "Command" | "Stack";

export interface GameState {
  active_player: PlayerId;
  battlefield: CardId[];
  cards: {
    [k: string]: CardEnt;
  };
  command: CardId[];
  exile: CardId[];
  extra_turns: PlayerId[];
  land_play_limit: number;
  lands_played_this_turn: number;
  mana: EntMapFor_ManaIdAnd_Mana;
  outcome: GameOutcome;
  phase?: Phase | null;
  phases: Phase[];
  player: PlayerId;
  players: {
    [k: string]: PlayerView;
  };
  priority: PlayerId;
  stack: CardId[];
  subphase?: Subphase | null;
  subphases: Subphase[];
  turn_order: PlayerId[];
  [k: string]: unknown;
}
export interface CardEnt {
  abilities: Ability[];
  already_dealt_damage: boolean;
  art_url?: string | null;
  attacking?: TargetId | null;
  blocked: CardId[];
  blocking: CardId[];
  controller?: PlayerId | null;
  costs: Cost[];
  damaged: number;
  effect: Clause[];
  ent_type: EntType;
  etb_this_cycle: boolean;
  known_to: PlayerId[];
  name: string;
  owner: PlayerId;
  printed_name: string;
  pt?: PT | null;
  subtypes: Subtypes;
  supertypes: Supertypes;
  tapped: boolean;
  types: Types;
  [k: string]: unknown;
}
export interface ActivatedAbility {
  costs: Cost[];
  effect: Clause[];
  keyword?: KeywordAbility | null;
  [k: string]: unknown;
}
export interface TriggeredAbility {
  [k: string]: unknown;
}
export interface StaticAbility {
  keyword?: KeywordAbility | null;
  [k: string]: unknown;
}
export interface PT {
  power: number;
  toughness: number;
  [k: string]: unknown;
}
export interface Subtypes {
  advisor: boolean;
  aetherborn: boolean;
  ally: boolean;
  angel: boolean;
  antelope: boolean;
  ape: boolean;
  archer: boolean;
  archon: boolean;
  army: boolean;
  artificer: boolean;
  assassin: boolean;
  assemblyworker: boolean;
  atog: boolean;
  aurochs: boolean;
  avatar: boolean;
  azra: boolean;
  badger: boolean;
  barbarian: boolean;
  bard: boolean;
  basilisk: boolean;
  bat: boolean;
  bear: boolean;
  beast: boolean;
  beeble: boolean;
  beholder: boolean;
  berserker: boolean;
  bird: boolean;
  blinkmoth: boolean;
  boar: boolean;
  bringer: boolean;
  brushwagg: boolean;
  camarid: boolean;
  camel: boolean;
  caribou: boolean;
  carrier: boolean;
  cat: boolean;
  centaur: boolean;
  cephalid: boolean;
  chimera: boolean;
  citizen: boolean;
  cleric: boolean;
  cockatrice: boolean;
  construct: boolean;
  coward: boolean;
  crab: boolean;
  crocodile: boolean;
  cyclops: boolean;
  dauthi: boolean;
  demigod: boolean;
  demon: boolean;
  deserter: boolean;
  devil: boolean;
  dinosaur: boolean;
  djinn: boolean;
  dog: boolean;
  dragon: boolean;
  drake: boolean;
  dreadnought: boolean;
  drone: boolean;
  druid: boolean;
  dryad: boolean;
  dwarf: boolean;
  efreet: boolean;
  egg: boolean;
  elder: boolean;
  eldrazi: boolean;
  elemental: boolean;
  elephant: boolean;
  elf: boolean;
  elk: boolean;
  endcreaturemarker: boolean;
  eye: boolean;
  faerie: boolean;
  ferret: boolean;
  fish: boolean;
  flagbearer: boolean;
  forest: boolean;
  fox: boolean;
  fractal: boolean;
  frog: boolean;
  fungus: boolean;
  gargoyle: boolean;
  germ: boolean;
  giant: boolean;
  gnoll: boolean;
  gnome: boolean;
  goat: boolean;
  goblin: boolean;
  god: boolean;
  golem: boolean;
  gorgon: boolean;
  graveborn: boolean;
  gremlin: boolean;
  griffin: boolean;
  hag: boolean;
  halfling: boolean;
  hamster: boolean;
  harpy: boolean;
  hellion: boolean;
  hippo: boolean;
  hippogriff: boolean;
  homarid: boolean;
  homunculus: boolean;
  horror: boolean;
  horse: boolean;
  human: boolean;
  hydra: boolean;
  hyena: boolean;
  illusion: boolean;
  imp: boolean;
  incarnation: boolean;
  inkling: boolean;
  insect: boolean;
  island: boolean;
  jackal: boolean;
  jellyfish: boolean;
  juggernaut: boolean;
  kavu: boolean;
  kirin: boolean;
  kithkin: boolean;
  knight: boolean;
  kobold: boolean;
  kor: boolean;
  kraken: boolean;
  lamia: boolean;
  lammasu: boolean;
  leech: boolean;
  leviathan: boolean;
  lhurgoyf: boolean;
  licid: boolean;
  lizard: boolean;
  manticore: boolean;
  masticore: boolean;
  mercenary: boolean;
  merfolk: boolean;
  metathran: boolean;
  minion: boolean;
  minotaur: boolean;
  mole: boolean;
  monger: boolean;
  mongoose: boolean;
  monk: boolean;
  monkey: boolean;
  moonfolk: boolean;
  mountain: boolean;
  mouse: boolean;
  mutant: boolean;
  myr: boolean;
  mystic: boolean;
  naga: boolean;
  nautilus: boolean;
  nephilim: boolean;
  nightmare: boolean;
  nightstalker: boolean;
  ninja: boolean;
  noble: boolean;
  noggle: boolean;
  nomad: boolean;
  nymph: boolean;
  octopus: boolean;
  ogre: boolean;
  ooze: boolean;
  orb: boolean;
  orc: boolean;
  orgg: boolean;
  otter: boolean;
  ouphe: boolean;
  ox: boolean;
  oyster: boolean;
  pangolin: boolean;
  peasant: boolean;
  pegasus: boolean;
  pentavite: boolean;
  pest: boolean;
  phelddagrif: boolean;
  phoenix: boolean;
  phyrexian: boolean;
  pilot: boolean;
  pincher: boolean;
  pirate: boolean;
  plains: boolean;
  plant: boolean;
  praetor: boolean;
  prism: boolean;
  processor: boolean;
  rabbit: boolean;
  ranger: boolean;
  rat: boolean;
  rebel: boolean;
  reflection: boolean;
  rhino: boolean;
  rigger: boolean;
  rogue: boolean;
  sable: boolean;
  salamander: boolean;
  samurai: boolean;
  sand: boolean;
  saproling: boolean;
  satyr: boolean;
  scarecrow: boolean;
  scion: boolean;
  scorpion: boolean;
  scout: boolean;
  sculpture: boolean;
  serf: boolean;
  serpent: boolean;
  servo: boolean;
  shade: boolean;
  shaman: boolean;
  shapeshifter: boolean;
  shark: boolean;
  sheep: boolean;
  siren: boolean;
  skeleton: boolean;
  slith: boolean;
  sliver: boolean;
  slug: boolean;
  snake: boolean;
  soldier: boolean;
  soltari: boolean;
  spawn: boolean;
  specter: boolean;
  spellshaper: boolean;
  sphinx: boolean;
  spider: boolean;
  spike: boolean;
  spirit: boolean;
  splinter: boolean;
  sponge: boolean;
  squid: boolean;
  squirrel: boolean;
  starfish: boolean;
  surrakar: boolean;
  survivor: boolean;
  swamp: boolean;
  tentacle: boolean;
  tetravite: boolean;
  thalakos: boolean;
  thopter: boolean;
  thrull: boolean;
  tiefling: boolean;
  treefolk: boolean;
  trilobite: boolean;
  triskelavite: boolean;
  troll: boolean;
  turtle: boolean;
  unicorn: boolean;
  vampire: boolean;
  vedalken: boolean;
  viashino: boolean;
  volver: boolean;
  wall: boolean;
  warlock: boolean;
  warrior: boolean;
  weird: boolean;
  werewolf: boolean;
  whale: boolean;
  wizard: boolean;
  wolf: boolean;
  wolverine: boolean;
  wombat: boolean;
  worm: boolean;
  wraith: boolean;
  wurm: boolean;
  yeti: boolean;
  zombie: boolean;
  zubera: boolean;
  [k: string]: unknown;
}
export interface Supertypes {
  basic: boolean;
  legendary: boolean;
  snow: boolean;
  world: boolean;
  [k: string]: unknown;
}
export interface Types {
  artifact: boolean;
  creature: boolean;
  enchantment: boolean;
  instant: boolean;
  land: boolean;
  planeswalker: boolean;
  sorcery: boolean;
  [k: string]: unknown;
}
export interface EntMapFor_ManaIdAnd_Mana {
  ents: {
    [k: string]: Mana;
  };
  [k: string]: unknown;
}
export interface Mana {
  color: Color;
  restriction?: ManaRestriction | null;
  [k: string]: unknown;
}
export interface ManaRestriction {
  [k: string]: unknown;
}
export interface PlayerView {
  graveyard: CardId[];
  hand: CardId[];
  library: CardId[];
  life: number;
  mana_pool: ManaId[];
  max_handsize: number;
  name: string;
  [k: string]: unknown;
}
export interface AskPairABFor_TargetId {
  a: {
    /**
     * @minItems 2
     * @maxItems 2
     */
    [k: string]: [number, number];
  };
  b: TargetId[];
  [k: string]: unknown;
}
export interface AskPairABFor_CardId {
  a: {
    /**
     * @minItems 2
     * @maxItems 2
     */
    [k: string]: [number, number];
  };
  b: CardId[];
  [k: string]: unknown;
}
export interface AskSelectNFor_CardId {
  ents: CardId[];
  max: number;
  min: number;
  [k: string]: unknown;
}
export interface AskSelectNFor_Action {
  ents: Action[];
  max: number;
  min: number;
  [k: string]: unknown;
}
export interface CastingOption {
  costs: Cost[];
  filter: ActionFilter;
  player: PlayerId;
  source_card: CardId;
  zone: Zone;
  [k: string]: unknown;
}
