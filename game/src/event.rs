use std::collections::HashMap;

use crate::game::{Phase, Subphase};
use common::{
    counters::Counter,
    entities::{CardId, PlayerId, TargetId},
    spellabil::Clause,
    zones::Zone,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DamageReason {
    Combat,
    SpellAbility(CardId),
}
//This will be wrapped when resolving to prevent
//replacement effects from triggering twice
#[derive(Clone, Debug)]
pub enum Event {
    Draw {
        player: PlayerId,
    },
    Damage {
        amount: i64,
        target: TargetId,
        source: CardId,
        reason: DamageReason,
    },
    Destroy {
        perms: Vec<CardId>,
    },
    Discard {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    Block {
        blocker: CardId,
    },
    Blocked {
        attacker: CardId,
    },
    BlockedBy {
        attacker: CardId,
        blocker: CardId,
    },
    Cast {
        player: PlayerId,
        spell: CardId,
    },
    Activate {
        controller: PlayerId,
        ability: CardId,
    },
    MoveZones {
        ents: Vec<CardId>,
        origin: Option<Zone>,
        dest: Zone,
    },
    Lose {
        player: PlayerId,
    },
    Tap {
        ent: CardId,
    },
    Subphase {
        subphase: Subphase,
    },
    PlayLand {
        player: PlayerId,
        land: CardId,
    },
    Phase {
        phase: Phase,
    },
    Turn {
        player: PlayerId,
        extra: bool,
    },
    Untap {
        ent: CardId,
    },
    GainLife {
        player: PlayerId,
        amount: i64,
    },
    TriggeredAbil {
        event: Box<EventResult>,
        source: CardId,
        effect: Vec<Clause>,
    },
    PutCounter {
        affected: TargetId,
        counter: Counter,
        quantity: i64,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct MoveZonesResult {
    pub oldent: CardId,
    pub newent: Option<CardId>,
    pub source: Option<Zone>,
    pub dest: Zone,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventResult {
    Draw(CardId),
    MoveZones(Vec<MoveZonesResult>),
    Tap(CardId),
    Untap(CardId),
    Attacks(HashMap<CardId, TargetId>),
}
