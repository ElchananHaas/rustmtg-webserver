use crate::{actions::StackActionOption, entities::CardId};
use mtg_log_macro::MTGLoggable;

struct GameContext{

}

trait MTGLog{
    type LogType;
    fn log(&self, game_context: &GameContext) -> Self::LogType;
}
#[derive(Clone, Debug)]
pub enum LogEntry {
    PermEntry {
        id: CardId,
        name: String,
        event: LogPermEntry,
    },
}

#[derive(Clone, Debug)]
pub enum LogPermEntry {
    DiesFromZeroOrLessToughness,
    DestroyFromDamage,
    DetachedEnchantmentDies,
    EnchantmentFallsOff,
    CastFailedFromRestriction,
    ManaCostNotPaid,
    Cast(StackActionOption),
}
