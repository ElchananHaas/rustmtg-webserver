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
            Affected::Target { target } => {
                if let Some(x) = target {
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
                    self.draw(controller).await;
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
        }
    }
}
