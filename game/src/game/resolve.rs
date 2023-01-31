use super::*;
use common::ability::{StaticAbility, StaticAbilityEffect};
use common::spellabil::{ContDuration, NumberComputer};
use common::token_attribute::TokenAttribute;

impl Game {
    pub fn compute_number(&self, id: CardId, computer: &NumberComputer) -> i64 {
        match computer {
            NumberComputer::NumPermanents(constraints) => {
                let mut count = 0;
                'outer: for perm in &self.battlefield {
                    for x in constraints{
                        if !self.passes_constraint(x, id, (*perm).into()){
                            continue 'outer;
                        }
                    }
                    count += 1;
                }
                count
            }
        }
    }
    pub async fn resolve(&mut self, id: CardId) {
        let effects;
        let types;
        if let Some(ent) = self.cards.get(id) {
            effects = ent.effect.clone();
            types = ent.types.clone();
        } else {
            return;
        }
        for effect in effects {
            self.resolve_clause(effect, id).await;
        }
        let dest = if types.is_instant() || types.is_sorcery() {
            Zone::Graveyard
        } else {
            Zone::Battlefield
        };
        self.move_zones(id, Zone::Stack, dest).await;
    }
    pub fn calculate_affected(
        &self,
        id: CardId,
        affected: &Affected,
        constraints: &Vec<PermConstraint>,
    ) -> Vec<TargetId> {
        let affected: Vec<TargetId> = match affected {
            Affected::Controller => if let Some(card)=self.cards.get(id){
                vec![card.get_controller().into()]
            } else {
                vec![]
            },
            Affected::Target(target) => {
                if let Some(x) = *target {
                    vec![x]
                } else {
                    vec![]
                }
            }
            Affected::ManuallySet(x) => {
                if let Some(x) = *x {
                    vec![x]
                } else {
                    vec![]
                }
            },
            Affected::Cardname => if let Some(card)=self.cards.get(id)
            && let Some(source)=card.source_of_ability{
                vec![source.into()]
            } else {
                vec![]
            },

        };
        return affected
            .into_iter()
            .filter(|&target| {
                constraints
                    .into_iter()
                    .all(|constraint| self.passes_constraint(constraint, id, target))
            })
            .collect();
    }
    #[async_recursion]
    #[must_use]
    async fn resolve_clause(&mut self, clause: Clause, id: CardId) {
        let affected: Vec<TargetId> =
            self.calculate_affected(id, &clause.affected, &clause.constraints);
        if affected.len() == 0 {
            return;
        }
        match clause.effect {
            ClauseEffect::AddMana(manas) => {
                for aff in affected {
                    if let TargetId::Player(pl) = aff {
                        for &mana in &manas {
                            self.add_mana(pl, mana).await;
                        }
                    }
                }
            }
            ClauseEffect::GainLife(amount) => {
                for aff in affected {
                    if let TargetId::Player(pl) = aff {
                        self.gain_life(pl, amount).await;
                    }
                }
            }
            ClauseEffect::DrawCard => {
                for aff in affected {
                    if let TargetId::Player(pl) = aff {
                        self.draw(pl).await;
                    }
                }
            }
            ClauseEffect::Destroy => {
                for aff in affected {
                    if let TargetId::Card(card) = aff {
                        self.destroy(card).await;
                    }
                }
            }
            ClauseEffect::ExileBattlefield => {
                for aff in affected {
                    if let TargetId::Card(card) = aff {
                        self.exile(card, Zone::Battlefield).await;
                    }
                }
            }
            ClauseEffect::Compound(clauses) => {
                for mut subclause in clauses {
                    subclause.affected = clause.affected; //Propagate target to subclause
                    self.resolve_clause(subclause, id).await;
                }
            }
            ClauseEffect::SetTargetController(clause) => {
                for aff in affected {
                    let mut clause = *clause.clone();
                    if let TargetId::Card(affected)=aff
                    && let Some(controller)=self.get_controller(affected){
                        clause.affected=Affected::ManuallySet(Some(controller.into()));
                        self.resolve_clause(clause, id).await;
                    }
                }
            }
            ClauseEffect::MultClause(inner_effect, computer) => {
                let multiplier = self.compute_number(id, &computer);
                for _ in 0..multiplier {
                    let new_clause = Clause {
                        effect: *inner_effect.clone(),
                        affected: clause.affected.clone(),
                        constraints: clause.constraints.clone(),
                    };
                    self.resolve_clause(new_clause, id).await;
                }
            }
            ClauseEffect::CreateToken(attributes) => {
                for aff in affected {
                    println!("creating token");
                    println!("{:?} : \n {:?}", attributes.clone(), aff);
                    if let TargetId::Player(affected) = aff {
                        let mut ent = CardEnt::default();
                        ent.owner = affected;
                        ent.controller = Some(affected);
                        ent.etb_this_cycle = true;
                        ent.ent_type = EntType::TokenCard;
                        for attribute in attributes.clone() {
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
                        let (id, _ent) = self.cards.insert(ent);
                        let results = self
                            .handle_event(Event::MoveZones {
                                ent: id,
                                origin: None,
                                dest: Zone::Battlefield,
                            })
                            .await;
                        println!("zone move results: {:?}", results);
                    }
                }
            }
            ClauseEffect::UntilEndTurn(conteffect) => {
                let cont_effect = Continuous {
                    affected: clause.affected,
                    effect: conteffect,
                    constraints: clause.constraints.clone(),
                    duration: ContDuration::EndOfTurn,
                    source: id,
                };
                self.cont_effects.push(cont_effect);
            }
        }
    }
}