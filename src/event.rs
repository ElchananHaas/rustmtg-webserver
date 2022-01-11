use crate::game::{Game, Phase, Subphase, Zone};
use hecs::Entity;

//An event tagged with replacement effects already applied to it
#[derive(Clone, Debug)]
pub struct TagEvent {
    pub event: Event,
    pub replacements: Vec<i32>,
}
//This will be wrapped when resolving to prevent
//replacement effects from triggering twice
#[derive(Clone, Debug)]
pub enum Event {
    Draw {
        player: Entity,
    },
    Cast {
        player: Entity,
        spell: Entity,
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
