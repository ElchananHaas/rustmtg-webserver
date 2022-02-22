use std::{collections::HashSet, fmt, num::NonZeroU32, sync::Arc};


use crate::{
    ability::{Ability, AbilityType},
    cost::Cost,
    game::Game,
    mana::{Color, ManaCostSymbol}, entities::CardId,
};

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
    pub fn activated_ability(mut self, cost: Vec<Cost>, mana_ability: bool) -> Ability {
        Ability {
            mana_ability,
            abil: AbilityType::Activated {
                cost,
                effect: self.clauses,
            },
            keyword: self.keyword,
        }
    }
    pub fn build(mut self) -> Vec<Clause> {
        self.clauses
    }
}
pub enum Clause {
    Effect {
        effect: ClauseEffect,
    },
    Target {
        targets: Targets,
        effect: TargetClauseEffect,
    },
}

pub enum ClauseEffect {
    AddMana(Vec<ManaCostSymbol>),
}
impl ClauseEffect {
    pub async fn run(&self, game: &mut Game, ent: CardId) {
        match self {
            Self::AddMana(manas) => {
                for mana in manas {
                    if let Ok(controller) = game.get_controller(ent) {
                        let _ = game.add_mana(controller, *mana).await;
                    }
                }
            }
        }
    }
}
pub enum TargetClauseEffect {}
impl TargetClauseEffect {
    pub async fn run(&self, game: &mut Game, ent: CardId) {}
}

pub enum ChosenClause {
    Effect {
        //The entity that this clause is a part of
        effect: Arc<dyn Fn(&mut Game, CardId)>,
    },
    Target {
        targets: ChosenTargets,
        effect: Arc<dyn Fn(&mut Game, ChosenTargets)>,
    },
}

pub struct Targets {
    num: NonZeroU32, //Ensure there is always at least 1 target, or
    //this clause shouldn't be chosen
    valid: Arc<dyn Fn(&Game, CardId) -> bool + Send + Sync>,
}
pub struct ChosenTargets {
    valid: Arc<dyn Fn(&Game, CardId) -> bool + Send + Sync>,
    targets: HashSet<CardId>,
}
