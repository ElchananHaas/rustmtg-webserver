use std::{fmt, collections::HashSet, num::NonZeroU32};

use hecs::Entity;

use crate::game::Game;
//use crate::carddb::CardBuilder;
pub type AbilBuildType = fn(&mut SpellAbilBuilder) -> &mut SpellAbilBuilder;
//origin entity, target entity
pub type TargetSpellAbliType=fn(&mut Game, Entity, Entity) -> ();
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeywordAbility {
    FirstStrike,
    Haste,
    Vigilance,
    DoubleStrike,
}

#[derive(Debug)]
pub struct TriggeredAbility {}
pub struct ActivatedAbility {
    mana_ability: bool,
    effect: AbilBuildType,
    keyword: Option<KeywordAbility>,
}
impl fmt::Debug for ActivatedAbility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActivatedAbility")
            .field("mana_ability", &self.mana_ability)
            .finish()
    }
}
#[derive(Clone, Debug)]
pub struct StaticAbility {}
#[derive(Debug)]
pub enum Ability {
    Activated(ActivatedAbility),
    Triggered(TriggeredAbility),
    Static(StaticAbility),
}
impl Ability {
    pub fn keyword(&self) -> Option<KeywordAbility> {
        //TODO!
        todo!("Can't tell if ability is keyword yet!");
    }
}

#[derive(Default)]
pub struct SpellAbilBuilder {
    pub clauses: Vec<Clause>,
    pub keyword: Option<KeywordAbility>,
}
impl SpellAbilBuilder{
    pub fn new()->Self{
        Self::default()
    }
    pub fn clause(&mut self, effect: fn(&mut Game, Entity) -> () )-> &mut Self{
        self.clauses.push(Clause::Effect{effect});
        self
    }
    pub fn target_clause(&mut self, targets:Targets,effect: TargetSpellAbliType )-> &mut Self{
        self.clauses.push(Clause::Target{targets,effect});
        self
    }
    pub fn build(mut self)->Vec<Clause>{
        self.clauses
    }
}
pub enum Clause {
    Effect {
        //The entity that this clause is a part of
        effect: fn(&mut Game, Entity) -> (),
    },
    Target {
        targets:Targets,
        effect: fn(&mut Game, Entity, Entity) -> (),
    },
}

pub enum ChosenClause {
    Effect {
        //The entity that this clause is a part of
        effect: fn(&mut Game, Entity) -> (),
    },
    Target {
        targets:ChosenTargets,
        effect: fn(&mut Game, ChosenTargets) -> (),
    },
}

pub struct Targets{
    num:NonZeroU32,//Ensure there is always at least 1 target, or
    //this clause shouldn't be chosen 
    valid:fn(&Game, Entity) -> bool
}
pub struct ChosenTargets{
    valid:fn(&Game, Entity) -> bool,
    targets:HashSet<Entity>,
}