use mtg_log_macro::MTGLoggable;

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
#[cfg(test)]
mod tests {
    use super::*;
}
