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
        attackers: Vec<CardId>,
    },
    Activate {
        controller: PlayerId,
        ability: CardId,
    },
    MoveZones {
        ent: CardId,
        origin: Zone,
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
        extra: bool,
        player: PlayerId,
    },
    Untap {
        ent: CardId,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum EventResult {
    Draw(CardId),
    Cast(CardId),
    Activate(CardId),
    MoveZones {
        oldent: CardId,
        newent: Option<CardId>,
        dest: Zone,
    },
    Tap(CardId),
    Untap(CardId),
}
