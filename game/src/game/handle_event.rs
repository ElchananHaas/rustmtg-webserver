mod combat;
mod phase_event;

use crate::{event::MoveZonesResult, game::*};
use async_recursion::async_recursion;
use common::{
    ability::{AbilityTriggerType, Replacement, TriggeredAbility},
    card_entities::EntType,
};

impl Game {
    /*
    This function should have stages, starting with prevention effects and
    going on to replacments, then finally
    the event can be handled
    */
    #[async_recursion]
    #[must_use]
    pub async fn handle_event(&mut self, event: Event) -> Vec<EventResult> {
        let mut results: Vec<EventResult> = Vec::new();
        let mut events: Vec<Event> = vec![event];
        let mut trigger_ents = self.battlefield.clone();
        loop {
            let event: Event = match events.pop() {
                Some(x) => x,
                None => {
                    //This fires all triggers once all events have happened.
                    for ent in &self.battlefield {
                        trigger_ents.add(*ent);
                    }
                    for result in &results {
                        self.fire_triggers(&trigger_ents, result).await;
                    }
                    return results;
                }
            };
            //Handle prevention effects
            if !self.allow_event(&event) {
                continue;
            }
            match self.replacements(&event).await {
                None => {}
                Some(mut replacements) => {
                    events.append(&mut replacements);
                    continue;
                }
            }
            //Handle prevention, replacement
            //By the time the loop reaches here, the game is ready to
            //Execute the event. No more prevention/replacement effects
            //At this point
            match event {
                Event::PutCounter { affected, counter, quantity }=>{
                    match affected{
                        TargetId::Card(cardid)=>{
                            if let Some(card)=self.cards.get_mut(cardid){
                                for _ in 0..quantity{
                                    card.counters.push(counter);
                                }
                            }
                        },
                        TargetId::Player(playerid)=>{
                            if let Some(pl)=self.players.get_mut(playerid){
                                for _ in 0..quantity{
                                    pl.counters.push(counter);
                                }
                            }
                        }
                    }
                }
                Event::GainLife { player, amount } =>{
                    if amount>0 && let Some(pl)=self.players.get_mut(player){
                        pl.life+=amount;
                    }
                }
                Event::Destroy { perms } => {
                    Game::add_event(
                        &mut events,
                        Event::MoveZones {
                            ents: perms,
                            origin: Some(Zone::Battlefield),
                            dest: Zone::Graveyard,
                        },
                    );
                }
                //The assigning as a blocker happens during the two-part block trigger
                Event::Damage {
                    amount,
                    target,
                    source,
                    reason: _,
                } => {
                    self.handle_damage(amount, target, source).await;
                }
                Event::Block { blocker: _ } => {}
                Event::BlockedBy { attacker, blocker } => {
                    self.blocked_by(&mut events, attacker, blocker).await;
                }
                Event::Blocked { attacker: _ } => {}
                Event::Discard {
                    player: _,
                    cards,
                } => {
                    Game::add_event(
                        &mut events,
                        Event::MoveZones {
                            ents: cards,
                            origin: Some(Zone::Hand),
                            dest: Zone::Graveyard,
                        },
                    );
                }
                Event::PlayLand { player: _, land } => {
                    if let Some(zone) = self.locate_zone(land) {
                        self.lands_played_this_turn += 1;
                        Game::add_event(
                            &mut events,
                            Event::MoveZones {
                                ents: vec![land],
                                origin: Some(zone),
                                dest: Zone::Battlefield,
                            },
                        )
                    }
                }
                Event::Turn { player, extra: _ } => {
                    self.active_player = player;
                    println!("starting turn");
                    self.phases.extend(
                        [
                            Phase::Begin,
                            Phase::FirstMain,
                            Phase::Combat,
                            Phase::SecondMain,
                            Phase::Ending,
                        ]
                        .iter(),
                    );
                }
                Event::Subphase { subphase } => {
                    self.subphase(&mut results, &mut events, subphase).await;
                }
                Event::Phase { phase } => {
                    self.phase(&mut events, phase).await;
                }
                //Handle already being tapped as prevention effect
                Event::Tap { ent } => {
                    if !self.battlefield.contains(&ent) {
                        continue;
                    }
                    self.cards.get_mut(ent).map(|card| {
                        if !card.tapped {
                            results.push(EventResult::Tap(ent));
                            card.tapped = true;
                        }
                    });
                }
                //Handle already being untapped as prevention effect
                Event::Untap { ent } => {
                    if !self.battlefield.contains(&ent) {
                        continue;
                    }
                    self.cards.get_mut(ent).map(|card| {
                        if card.tapped {
                            results.push(EventResult::Untap(ent));
                            card.tapped = false;
                        }
                    });
                }
                Event::Draw { player } => {
                    if let Some(pl) = self.players.get_mut(player) {
                        match pl.library.last() {
                            Some(card) => {
                                Game::add_event(
                                    &mut events,
                                    Event::MoveZones {
                                        ents: vec![*card],
                                        origin: Some(Zone::Library),
                                        dest: Zone::Hand,
                                    },
                                );
                                results.push(EventResult::Draw(*card));
                            }
                            None => Game::add_event(&mut events, Event::Lose { player: player }),
                        }
                    }
                }
                Event::Cast {
                    player: _,
                    spell: _,
                } => {
                    //The spell has already had costs/modes chosen.
                    //this is just handling triggered abilities
                    //So there is nothing to do here.
                    //Spells are handled differently from other actions
                    //Because of the rules complexity
                }
                Event::Activate {
                    controller: _,
                    ability: _,
                } => {
                    //Similar to spell casting
                }
                Event::Lose { player } => {
                    //TODO add in the logic to have the game terminate such as setting winners
                    todo!();
                }
                Event::MoveZones { ents, origin, dest } => {
                    self.movezones(&mut results, &mut events, ents, origin, dest)
                        .await;
                },
                Event::TriggeredAbil { event: _, source,effect } => {
                    let mut new_card=CardEnt::default();
                    new_card.effect=effect;
                    new_card.source_of_ability=Some(source);
                    if let Some(pl)=self.get_controller(source){
                        new_card.owner=pl;
                        new_card.set_controller(Some(pl));
                    }
                    new_card.ent_type=EntType::TriggeredAbility;
                    new_card.printed=Some(Box::new(new_card.clone()));
                    let (id,card)=self.cards.insert(new_card);
                    self.stack.push(id);
                    let controller=card.get_controller();
                    self.log(Entry::TriggeredAbil(id));
                    self.send_state().await;
                    let _=self.select_targets(controller, id).await;
                    //TODO check if there is a valid target assignment,
                    //because if so the player must take it.
                }
            }
        }
    }
    async fn replacement_for_cardid(
        &self,
        event: &Event,
        sourceid: CardId,
        abil: &Replacement,
    ) -> Option<(Vec<Event>, Vec<Clause>)> {
        match abil {
            Replacement::ZoneMoveReplacement {
                constraints,
                trigger,
                new_effect,
            } => {
                if let Event::MoveZones { ents, origin, dest } = event {
                    let mut keep = Vec::new();
                    let mut replace = Vec::new();
                    for entid in ents {
                        if constraints
                            .iter()
                            .all(|c| self.passes_constraint(c, sourceid, (*entid).into()))
                            && trigger.origin.map_or(true, |zone| Some(zone) == *origin)
                            && trigger.dest.map_or(true, |zone| zone == *dest)
                        {
                            replace.push(*entid)
                        } else {
                            keep.push(*entid)
                        }
                    }
                    if replace.len() == 0 {
                        return None;
                    } else {
                        let mut new_effect = new_effect.clone();
                        if let Affected::ManuallySet(_) = new_effect.affected {
                            let replace = replace.into_iter().map(|x| x.into()).collect();
                            new_effect.affected = Affected::ManuallySet(replace);
                        }
                        let res = (
                            vec![Event::MoveZones {
                                ents: keep,
                                origin: *origin,
                                dest: *dest,
                            }],
                            vec![new_effect],
                        );
                        return Some(res);
                    }
                }
            }
        }
        None
    }
    async fn replacements(&mut self, event: &Event) -> Option<Vec<Event>> {
        if let Some((events, clauses, cardid)) = self.replacements_h(event).await {
            for clause in clauses {
                self.resolve_clause(clause, cardid).await;
            }
            Some(events)
        } else {
            return None;
        }
    }
    async fn replacements_h(&mut self, event: &Event) -> Option<(Vec<Event>, Vec<Clause>, CardId)> {
        for cardid in &self.battlefield {
            if let Some(card) = self.cards.get(*cardid) {
                for abil in &card.abilities {
                    if let Ability::Replacement(abil) = abil {
                        if let Some(replaced) = self
                            .replacement_for_cardid(event, *cardid, &abil.effect)
                            .await
                        {
                            return Some((replaced.0, replaced.1, *cardid));
                        }
                    }
                }
            }
        }
        None
    }
    fn allow_event(&self, event: &Event) -> bool {
        if let Event::Damage { amount:_, target, source, reason:_ }=event 
        && self.has_protection_from(*source, *target){
            return false;
        }
        true
    }

    fn triggers_for_abil(
        &self,
        abil: &TriggeredAbility,
        source_id: CardId,
        event: &EventResult,
    ) -> Vec<Event> {
        let trigger = &abil.trigger;
        let mut res = Vec::new();
        match &trigger.trigger {
            AbilityTriggerType::ZoneMove(trig) => {
                if let EventResult::MoveZones(results) = event {
                    for result in results {
                        let constrained_card=if result.source!=Some(Zone::Battlefield)
                        && let Some(newent)=result.newent{
                            newent
                        } else {
                            result.oldent
                        };
                        let to_fire = trig.origin.map_or(true, |x| Some(x) == result.source)
                            && trig.dest.map_or(true, |x| x == result.dest)
                            && trigger.constraint.iter().all(|c| {
                                self.passes_constraint(c, source_id, constrained_card.into())
                            });
                        if to_fire {
                            res.push(Event::TriggeredAbil {
                                event: Box::new(event.clone()),
                                source: source_id,
                                effect: abil.effect.clone(),
                            })
                        }
                    }
                }
            },
            AbilityTriggerType::Attacks => {
                if let EventResult::Attacks(attackers) = event {
                    let responsible=attackers.iter().filter(
                        |&(attacker,attacked)|
                        trigger.constraint.iter().all(|c|
                            self.passes_constraint(c, *attacker,* attacked)
                        )
                    ).collect::<Vec<_>>();
                    if responsible.len() > 0 {
                        res.push(Event::TriggeredAbil {
                            event: Box::new(event.clone()),
                            source: source_id,
                            effect: abil.effect.clone(),
                        })
                    }
                }
            }
        }
        res
    }
    async fn fire_triggers(&mut self, trigger_ents: &HashSetObj<CardId>, event: &EventResult) {
        let mut events: Vec<Event> = Vec::new();
        for cardid in trigger_ents {
            if let Some(card) = self.cards.get(*cardid) {
                for abil in &card.abilities {
                    if let Ability::Triggered(abil) = abil {
                        events.append(&mut self.triggers_for_abil(abil, *cardid, event));
                    }
                }
            }
        }
        for event in events {
            self.handle_event(event).await;
        }
    }
    async fn drain_mana_pools(&mut self) {
        //TODO handle effects that keep mana pools from draining
        for &player in &self.turn_order {
            if let Some(pl) = self.players.get_mut(player) {
                pl.mana_pool = HashSetObj::new();
            }
        }
    }

    async fn movezones(
        &mut self,
        results: &mut Vec<EventResult>,
        _events: &mut Vec<Event>,
        ents: Vec<CardId>,
        origin: Option<Zone>,
        dest: Zone,
    ) {
        let mut move_results = Vec::new();
        for ent in ents {
            if let Some(card)=self.cards.get_mut(ent)
        && let Some(owner)= self.players.get_mut(card.owner) {
            let removed = if let Some(origin)=origin{
            match origin {
                Zone::Exile => self.exile.remove(&ent),
                Zone::Command => self.command.remove(&ent),
                Zone::Battlefield => self.battlefield.remove(&ent),
                Zone::Hand => owner.hand.remove(&ent),
                Zone::Library => match owner.library.iter().position(|x| *x == ent) {
                    Some(i) => {
                        owner.library.remove(i);
                        true
                    }
                    None => {
                        false},
                },
                Zone::Graveyard => match owner.graveyard.iter().position(|x| *x == ent) {
                    Some(i) => {
                        owner.graveyard.remove(i);
                        true
                    }
                    None => false,
                },
                Zone::Stack => match self.stack.iter().position(|x| *x == ent) {
                    Some(i) => {
                        self.stack.remove(i);
                        true
                    }
                    None => false,
                },
            }}else{
                true
            };
            let real = card.ent_type == EntType::RealCard;
            if removed{
                if !real && origin.is_some(){
                    move_results.push(MoveZonesResult{
                        oldent: ent,
                        newent: None,
                        source: origin,
                        dest,
                    });
                    return;
                }else{
                    let newcard=if origin==Some(Zone::Stack){
                        card.clone()
                    }else{
                        let mut newcard=card.printed.as_ref().expect("set printed card").as_ref().clone();
                        newcard.owner=card.owner;
                        newcard.printed=Some(Box::new(newcard.clone()));
                        newcard
                    };
                    let (newent, newcard) = self.cards.insert(newcard);
                    match dest {
                        Zone::Exile | Zone::Stack | Zone::Command | Zone::Battlefield | Zone::Graveyard => {
                            newcard.known_to.extend(self.turn_order.iter());
                            //Public zone
                        }
                        Zone::Hand => {
                            newcard.known_to.insert(newcard.owner);
                        } //Shuffling will destroy all knowledge of cards in the library
                        _ => {}
                    }
                    match dest {
                        Zone::Exile => {
                            self.exile.insert(newent);
                        }
                        Zone::Command => {
                            self.command.insert(newent);
                        }
                        Zone::Battlefield => {
                            self.battlefield.insert(newent);
                            newcard.etb_this_cycle=true;
                        }
                        Zone::Hand => {
                            owner.hand.insert(newent);
                        }
                        //Handle inserting a distance from the top. Perhaps swap them afterwards?
                        Zone::Library => owner.library.push(newent),
                        Zone::Graveyard => owner.graveyard.push(newent),
                        Zone::Stack => self.stack.push(newent),
                    }
                    move_results.push(MoveZonesResult {
                        oldent: ent,
                        newent: Some(newent),
                        source: origin,
                        dest,
                    });
                }
            }
        };
        }
        results.push(EventResult::MoveZones(move_results))
    }

    //Add deathtouch and combat triggers
    async fn handle_damage(&mut self, amount: i64, target: TargetId, source: CardId) {
        if amount <= 0 {
            return;
        }
        match target {
            TargetId::Card(cardid) => {
                if let Some(card) = self.cards.get_mut(cardid) {
                    card.damaged += amount;
                }
            }
            TargetId::Player(playerid) => {
                if let Some(player) = self.players.get_mut(playerid) {
                    player.life -= amount;
                }
            }
        }
        if let Some(card)=self.cards.get(source)
        && card.has_keyword(KeywordAbility::Lifelink){
            self.handle_event(Event::GainLife { player: card.get_controller(), amount }).await;
        }
    }
    fn add_event(events: &mut Vec<Event>, event: Event) {
        events.push(event);
    }
}
