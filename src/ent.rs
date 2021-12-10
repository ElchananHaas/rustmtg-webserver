use crate::game::{EntID,PlayerID,Game,Color,Supertype,Cardtype};
use crate::subtypes::{Subtype};
use std::collections::HashSet;
use dyn_clone::DynClone;
//Entities include cards, tokens and activated abilties


#[derive(Clone,Copy)]
pub struct PT{
    pub power:i32,
    pub toughness:i32,
}


#[derive(Clone)]
pub struct Cost{
    pub white:i32,
    pub blue:i32,
    pub red:i32,
    pub black:i32,
    pub green:i32,
    pub generic:i32,
} //Should I split this into individual mana costs (for each color)?
//Or just use a big struct for all costs, probably easiest at the start.
//Don't put in too many layers of abstraction!

impl Cost{
    pub fn fromString(strcost:&str)->Self{
        unimplemented!("Add mana cost from string");
    }
}
#[derive(Clone)]
pub struct Targets{

}

pub trait Effect: DynClone{
    fn resove(&mut game:Game,&cost_paid:Option<Cost>,&targets:Option<Targets>)
}

dyn_clone::clone_trait_object!(Effect);

#[derive(Clone)]
pub struct ActivatedAbility{
}
#[derive(Clone)]
pub struct TriggereAbility{
}
#[derive(Clone)]
pub struct ContinuousAbility{
}

#[derive(PartialEq, Eq, Hash,Copy,Clone,Debug)]
pub enum KeywordAbility{
    Deathtouch,
    Defender,
    DoubleStrike,
    Enchant,
    Equip,
    FirstStrike,
    Flash,
    Flying,
    Haste,
    Hexproof,
    Indestructible,
    Lifelink,
    Menace,
    Reach,
    Trample,
    Vigilance
}
#[derive(Clone)]
pub struct Ability{
    keyword:Option<KeywordAbility>,
    ability:AbilityType
}
#[derive(Clone)]
pub enum AbilityType{
    Actived(ActivatedAbility),
    Triggered(TriggeredAbility),
    Continuous(ContinuousAbility),
}

#[derive(Clone)]
pub struct PermBody{
    pub abilties:List<Ability>
    pub pt:Option<PT>,
}
#[derive(Clone)]
pub struct StackBody{
    pub cost_paid:Option<Cost>,
    pub targets:Option<Targets>
    pub effect:Box<dyn Effect>,
}

#[derive(Clone)]
pub enum Body{
    Perm(PermBody),
    Stack(StackBody)
}
#[derive(Clone)]
pub struct Mode{
    pub name:Option<String>,
    pub cost:Option<Cost>,
    pub supertypes:HashSet<Supertype>,
    pub types:HashSet<Cardtype>,
    pub subtypes:HashSet<Subtype>,
    pub body:Body
}

#[derive(Clone)]
pub struct Ent{
    pub printed_modes: Vec<Mode>,
    pub modes: Vec<Mode>, //Front side/primary mode is first
}

impl Ent{
    pub power(&self)->Option<i32>{

    }
    pub toughness(&self)->Option<i32>{

    }
}