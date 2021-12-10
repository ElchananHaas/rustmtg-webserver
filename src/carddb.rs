use hecs::{World,Entity};
use anyhow::{Result};
pub struct CardDB{

}


impl CardDB{
    pub fn spawn_card(&self,ents:&mut World,card_name:&str)->Result<Entity>{
        unimplemented!();
    }
}