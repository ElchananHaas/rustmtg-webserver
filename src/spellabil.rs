use std::{collections::HashSet,  num::NonZeroU32, sync::Arc};

use serde_derive::Serialize;

use crate::{
    ability::{Ability, ActivatedAbility},
    cost::Cost,
    entities::CardId,
    game::Game,
    mana::{ ManaCostSymbol},
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
    pub fn activated_ability(mut self, cost: Vec<Cost>, mana_ability: bool) -> Ability {
        let activated = ActivatedAbility {
            cost,
            keyword: self.keyword,
            effect: self.clauses,
            mana_ability,
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
}
impl ClauseEffect {
    pub async fn run(&self, game: &mut Game, ent: CardId) {
        match self {
            Self::AddMana(manas) => {
                for mana in manas {
                    if let Some(controller) = game.get_controller(ent) {
                        let _ = game.add_mana(controller, *mana).await;
                    }
                }
            }
        }
    }
}
#[derive(Clone, Serialize)]
pub enum TargetClauseEffect {}
impl TargetClauseEffect {
    pub async fn run(&self, game: &mut Game, ent: CardId) {}
}

//TODO change this to an enum
#[derive(Clone, Serialize)]
pub struct Targets {
    num: NonZeroU32, //Ensure there is always at least 1 target, or
    //this clause shouldn't be chosen
    #[serde(skip_serializing)]
    valid: Arc<dyn Fn(&Game, CardId) -> bool + Send + Sync>,
}
