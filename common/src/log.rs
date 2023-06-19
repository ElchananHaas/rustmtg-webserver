use crate::{ entities::{CardId, PlayerId}, ent_maps::EntMap, card_entities::CardEnt};

pub struct GameContext<'a>{
    pub cards: &'a EntMap<CardId,CardEnt>,
}

pub trait MTGLog{
    type LogType;
    fn mtg_log(&self, game_context: &GameContext) -> Self::LogType;
}
#[allow(dead_code)] //Not dead, used for logging
#[derive(Clone,Debug)]
pub struct CardIdContext{
    card_id: CardId,
    name: Option<String>,
    controller: Option<PlayerId>,
}
impl MTGLog for CardId{
    type LogType = CardIdContext;
    fn mtg_log(&self, game_context: &GameContext) -> CardIdContext{
        let (name,controller) = if let Some(card)=game_context.cards.get(*self){
            (Some(card.name.clone()),Some(card.get_controller()))
        } else {
            (None,None)
        };
        CardIdContext{
            card_id: *self,
            name,
            controller
        }
    }
}
impl <T:MTGLog> MTGLog for Option<T>{
    type LogType = Option<<T as MTGLog>::LogType>;
    fn mtg_log(&self, game_context: &GameContext) -> Self::LogType{
        match self {
            Some(x) => {
                Some(x.mtg_log(game_context))
            },
            None => None
        }
    }
}
impl <T:MTGLog> MTGLog for Vec<T>{
    type LogType = Vec<<T as MTGLog>::LogType>;
    fn mtg_log(&self, game_context: &GameContext) -> Self::LogType{
        self.into_iter().map(|x| x.mtg_log(game_context)).collect::<Self::LogType>()
    }
}
impl <T:MTGLog> MTGLog for Box<T>{
    type LogType = Box<<T as MTGLog>::LogType>;
    fn mtg_log(&self, game_context: &GameContext) -> Self::LogType{
        let inner: &T=&*self;
        Box::new(inner.mtg_log(game_context))
    }
}

impl MTGLog for i64{
    type LogType = i64;
    fn mtg_log(&self, _game_context: &GameContext) -> Self::LogType{
        *self
    }
}
impl MTGLog for PlayerId{
    type LogType = PlayerId;
    fn mtg_log(&self, _game_context: &GameContext) -> Self::LogType{
        *self
    }
}