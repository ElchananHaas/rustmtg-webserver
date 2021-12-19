use hecs::{World,Entity};
use anyhow::{Result,bail};
use std::collections::HashSet;
use crate::carddb::CardDB;
pub struct GameBuilder{
    ents:World,
    turn_order:Vec<Entity>,
    active_player:Option<Entity>,
}
//Implement debug trait!
//Implement clone trait???
pub struct Game{
    ents:World,
    battlefield:HashSet<Entity>,
    exile:HashSet<Entity>,
    command:HashSet<Entity>,
    stack:Vec<Entity>,
    turn_order:Vec<Entity>,
    active_player:Entity
}



impl GameBuilder{
    pub fn new()->Self{
        GameBuilder{
            ents:World::new(),
            turn_order:Vec::new(),
            active_player:None,
        }
    }
    //If this function fails the game is corrupted
    //Potentialy fail game creation if player can't be added?
    pub fn add_player(&mut self,name:&str,db:&CardDB,card_names:&Vec<String>)->Result<Entity>{
        let mut cards=Vec::new();
        let name=PlayerName(name.to_owned());
        let hand=Hand(HashSet::new());
        let life=Life(20);
        let player:Entity=self.ents.spawn((name,hand,life));
        for cardname in card_names{
            let card:Entity=db.spawn_card(&mut self.ents,&cardname)?;
            self.ents.insert_one(card,Owner(player))?;
            cards.push(card);
        }
        let deck=Deck(cards);
        self.ents.insert_one(player,deck)?;
        self.turn_order.push(player);
        if self.active_player.is_none(){
            self.active_player=Some(player);
        }
        Ok(player)
    }
    pub fn build(self)->Result<Game>{
        let active_player=match self.active_player{
            Some(player)=>player,
            None=> {bail!("Active player must be set in game initilization");}
        };
        if self.turn_order.len()<2 {bail!("Game needs at least two players in initilization")};
        Ok(Game{
            ents:self.ents,
            battlefield:HashSet::new(),
            exile:HashSet::new(),
            command:HashSet::new(),
            stack:Vec::new(),
            turn_order:self.turn_order,
            active_player:active_player,
        })
    }
}
#[derive(Clone,Copy,Debug)]
pub struct Owner(Entity);
#[derive(Clone,Debug)]
pub struct PlayerName(String);
#[derive(Clone,Copy,Debug)]
pub struct Life(i32);
#[derive(Clone,Debug)]
pub struct Deck(Vec<Entity>);
#[derive(Clone,Debug)]
pub struct Hand(HashSet<Entity>);



