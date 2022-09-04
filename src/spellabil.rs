use std::{collections::HashSet, num::NonZeroU32, sync::Arc};

use serde_derive::Serialize;

use crate::{
    ability::{Ability, ActivatedAbility},
    cost::Cost,
    entities::CardId,
    game::Game,
    mana::ManaCostSymbol,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
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
impl SpellAbilBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clause(&mut self, effect: ClauseEffect) -> &mut Self {
        self.clauses.push(Clause::Effect { effect });
        self
    }
    pub fn target_clause(&mut self, targets: Targets, effect: TargetClauseEffect) -> &mut Self {
        self.clauses.push(Clause::Target { targets, effect });
        self
    }
    pub fn from_keyword(keyword: KeywordAbility) -> Self {
        let mut res = Self::default();
        res.keyword = Some(keyword);
        match keyword {
            _ => todo!(),
        };
        res
    }
    pub fn activated_ability(mut self, costs: Vec<Cost>) -> Ability {
        let activated = ActivatedAbility {
            costs,
            keyword: self.keyword,
            effect: self.clauses,
        };
        Ability::Activated(activated)
    }
    pub fn build(mut self) -> Vec<Clause> {
        self.clauses
    }
}
#[derive(Clone, Serialize)]
pub enum Clause {
    Effect {
        effect: ClauseEffect,
    },
    Target {
        targets: Targets,
        effect: TargetClauseEffect,
    },
}
#[derive(Clone, Serialize)]
pub enum ClauseEffect {
    AddMana(Vec<ManaCostSymbol>),
    DrawCard,
}

#[derive(Clone, Serialize)]
pub enum TargetClauseEffect {}

#[derive(Clone, Serialize)]
pub struct Targets {
    num: NonZeroU32, //Ensure there is always at least 1 target, or
    //this clause shouldn't be chosen
    valid: TargetsFilter,
}
#[derive(Clone, Serialize)]
pub enum TargetsFilter {}
