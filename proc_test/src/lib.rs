#![allow(dead_code)]
#![allow(unused_variables)]
use mtg_log_macro::MTGLoggable;

pub struct GameContext{
}

pub trait MTGLog{
    type LogType;
    fn mtg_log(&self, game_context: &GameContext) -> Self::LogType{
        panic!()
    }
}
impl MTGLog for i32{
    type LogType = ();
}
impl MTGLog for u64{
    type LogType = ();
}
impl MTGLog for usize{
    type LogType = ();
}
impl<T> MTGLog for Option<T>{
    type LogType = ();
}
#[derive(MTGLoggable)]
struct Test{
    x:i32,
    y:u64, 
}

#[derive(MTGLoggable)]
struct Unnamed(usize);

#[derive(MTGLoggable)]
pub enum Simple{
    Apple,
    Orange(i32)
}

#[derive(MTGLoggable)]
enum Adt{
    Apple{
        hello : i32
    },
    Orange
}

#[derive(MTGLoggable)]
pub struct ZoneMoveTrigger {
    //These both must match for the ability to trigger
    pub origin: Option<i32>,
    pub dest: Option<i32>,
}
#[derive(MTGLoggable)]
struct Unit;

