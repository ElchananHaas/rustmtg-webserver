use std::{fmt, collections::HashSet, num::NonZeroU32};

use hecs::Entity;

use crate::{game::Game, ability::{Ability, AbilityType}};

pub type SpellAbilBuildType = fn(&mut SpellAbilBuilder) -> &mut SpellAbilBuilder;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeywordAbility {
    FirstStrike,
    Haste,
    Vigilance,
    DoubleStrike,
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
    pub fn target_clause(&mut self, targets:Targets,effect: fn(&mut Game,Entity, ChosenTargets) -> () )-> &mut Self{
        self.clauses.push(Clause::Target{targets,effect});
        self
    }
    pub fn from_keyword(keyword:KeywordAbility)->Self{
        let mut res=Self::default();
        res.keyword=Some(keyword);
        match keyword{
            _=>todo!()
        };
        res 
    }
    pub fn activated_ability(mut self,mana_ability:bool)->Ability{
        Ability{
            mana_ability,
            abil: AbilityType::Activated{effect: self.clauses},
            keyword: self.keyword
        }
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
        effect: fn(&mut Game, Entity, ChosenTargets) -> (),
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