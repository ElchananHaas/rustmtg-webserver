use crate::{
    components::EntCore,
    game::{Game, Phase, Subphase, Zone},
};
use hecs::Entity;

//An event tagged with replacement effects already applied to it
#[derive(Clone, Debug)]
pub struct TagEvent {
    pub event: Event,
    pub replacements: Vec<i32>,
}
#[derive(Clone, Debug)]
pub enum DiscardCause {
    GameInternal,
    SpellAbility(Entity),
}
#[derive(Clone, Debug)]
pub enum DamageReason {
    Combat,
    SpellAbility(Entity),
}
//This will be wrapped when resolving to prevent
//replacement effects from triggering twice
#[derive(Clone, Debug)]
pub enum Event {
    Draw {
        player: Entity,
    },
    Damage{
        amount:i32,
        ent:Entity,
        source:Entity,
        reason:DamageReason,
    },
    Discard {
        player: Entity,
        card: Entity,
        cause: DiscardCause,
    },
    Block {
        blocker: Entity,
    },
    Blocked {
        attacker: Entity,
    },
    BlockedBy {
        attacker: Entity,
        blocker: Entity,
    },
    AttackUnblocked {
        attacker: Entity,
    },
    Cast {
        player: Entity,
        spell: Entity,
    },
    Attack {
        attackers: Vec<Entity>,
    },
    Activate {
        controller: Entity,
        ability: Entity,
    },
    MoveZones {
        ent: Entity,
        origin: Zone,
        dest: Zone,
    },
    Lose {
        player: Entity,
    },
    Tap {
        ent: Entity,
    },
    Subphase {
        subphase: Subphase,
    },
    Phase {
        phase: Phase,
    },
    Turn {
        extra: bool,
        player: Entity,
    },
    Untap {
        ent: Entity,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum EventResult {
    Draw(Entity),
    Cast(Entity),
    Activate(Entity),
    MoveZones {
        oldent: Entity,
        newent: Entity,
        dest: Zone,
    },
    Tap(Entity),
    Untap(Entity),
}
