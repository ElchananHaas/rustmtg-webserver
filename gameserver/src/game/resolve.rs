use crate::{
    ability::{StaticAbility, StaticAbilityEffect},
    token_builder::TokenAttribute,
};

use super::*;
impl Game {
    pub async fn resolve(&mut self, id: CardId) {
        let effects;
        let controller;
        let types;
        if let Some(ent) = self.cards.get(id) {
            effects = ent.effect.clone();
            controller = ent.get_controller();
            types = ent.types.clone();
        } else {
            return;
        }
        for effect in effects {
            self.resolve_clause(effect, id, controller).await;
        }
        let dest = if types.instant || types.sorcery {
            Zone::Graveyard
        } else {
            Zone::Battlefield
        };
        self.move_zones(id, Zone::Stack, dest).await;
    }
    #[async_recursion]
    #[must_use]
    async fn resolve_clause(&mut self, clause: Clause, id: CardId, controller: PlayerId) {
        let affected: TargetId = match clause.affected {
            Affected::Controller => controller.into(),
            Affected::Target(target) => {
                if let Some(x) = target {
                    x
                } else {
                    return;
                }
            }
            Affected::ManuallySet(x) => {
                if let Some(x) = x {
                    x
                } else {
                    return;
                }
            }
        };
        for constraint in clause.constraints {
            if !constraint.passes_constraint(self, affected) {
                return;
            }
        }
        match clause.effect {
            ClauseEffect::AddMana(manas) => {
                if let TargetId::Player(pl) = affected {
                    for mana in manas {
                        self.add_mana(pl, mana).await;
                    }
                }
            }
            ClauseEffect::GainLife(amount) => {
                if let TargetId::Player(pl) = affected {
                    self.gain_life(pl, amount).await;
                }
            }
            ClauseEffect::DrawCard => {
                if let TargetId::Player(pl) = affected {
                    self.draw(pl).await;
                }
            }
            ClauseEffect::Destroy => {
                if let TargetId::Card(card) = affected {
                    self.destroy(card).await;
                }
            }
            ClauseEffect::ExileBattlefield => {
                if let TargetId::Card(card) = affected {
                    self.exile(card, Zone::Battlefield).await;
                }
            }
            ClauseEffect::Compound(clauses) => {
                for mut subclause in clauses {
                    subclause.affected = clause.affected; //Propagate target to subclause
                    self.resolve_clause(subclause, id, controller).await;
                }
            }
            ClauseEffect::SetTargetController(clause) => {
                let mut clause = *clause;
                if let TargetId::Card(affected)=affected
                && let Some(controller)=self.get_controller(affected){
                    clause.affected=Affected::ManuallySet(Some(controller.into()));
                    self.resolve_clause(clause, id ,controller).await;
                }
            }
            ClauseEffect::CreateToken(attributes) => {
                println!("creating token");
                println!("{:?} : \n {:?}", attributes, affected);
                if let TargetId::Player(affected) = affected {
                    let mut ent = CardEnt::default();
                    ent.owner = affected;
                    ent.controller = Some(affected);
                    ent.etb_this_cycle = true;
                    ent.ent_type = EntType::TokenCard;
                    for attribute in attributes {
                        match attribute {
                            TokenAttribute::PT(pt) => {
                                ent.pt = Some(pt);
                            }
                            TokenAttribute::Type(t) => {
                                ent.types.add(t);
                            }
                            TokenAttribute::Subtype(t) => {
                                ent.subtypes.add(t);
                            }
                            TokenAttribute::HasColor(color) => {
                                ent.abilities.push(Ability::Static(StaticAbility {
                                    keyword: None,
                                    effect: StaticAbilityEffect::HasColor(color),
                                }));
                            }
                            TokenAttribute::Ability(abil) => {
                                ent.abilities.push(abil);
                            }
                        }
                    }
                    ent.printed = Some(Box::new(ent.clone()));
                    let (id, ent) = self.cards.insert(ent);
                    let results = self
                        .handle_event(Event::MoveZones {
                            ent: id,
                            origin: None,
                            dest: Zone::Battlefield,
                        })
                        .await; //This will need to be modified to use proper
                                //triggers/zonemove
                    println!("zone move results: {:?}", results);
                }
            }
        }
    }
}
