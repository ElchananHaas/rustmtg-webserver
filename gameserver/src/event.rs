use std::collections::HashMap;

use crate::{
    entities::{CardId, PlayerId, TargetId},
    game::{Phase, Subphase, Zone},
};

//An event tagged with replacement effects already applied to it
#[derive(Clone, Debug)]
pub struct TagEvent {
    pub event: Event,
    pub replacements: Vec<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiscardCause {
    GameInternal,
    SpellAbility(CardId),
}
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
        card: CardId,
    },
    Discard {
        player: PlayerId,
        card: CardId,
        cause: DiscardCause,
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
    AttackUnblocked {
        attacker: CardId,
    },
    Cast {
        player: PlayerId,
        spell: CardId,
    },
    Attack {
        attacks: HashMap<CardId, TargetId>,
    },
    Activate {
        controller: PlayerId,
        ability: CardId,
    },
    MoveZones {
        ent: CardId,
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
}
#[derive(Clone, Debug, PartialEq)]
pub enum EventResult {
    Draw(CardId),
    MoveZones {
        oldent: CardId,
        newent: Option<CardId>,
        source: Option<Zone>,
        dest: Zone,
    },
    Tap(CardId),
    Untap(CardId),
}
