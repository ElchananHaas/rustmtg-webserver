use crate::game::{Game,Zone};
use hecs::Entity;

//An event tagged with replacement effects already applied to it
#[derive(Clone, Debug)]
pub struct TagEvent{
    pub event:Event,
    pub replacements:Vec<i32>,
}
//This will be wrapped when resolving to prevent 
//replacement effects from triggering twice
#[derive(Clone, Debug)]
pub enum Event{
    Draw{player:Entity,controller:Option<Entity>},
    Cast{player:Entity,spell:Entity},
    Activate{player:Entity,ability:Entity},
    MoveZones{ent:Entity,origin:Zone,dest:Zone},
    Lose{player:Entity},
}

pub enum EventResult{
    Draw{card:Entity},
    Cast{spell:Entity},
    Activate{ability:Entity},
    MoveZones{ent:Entity,origin:Zone,dest:Zone}
}