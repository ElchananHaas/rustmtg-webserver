use crate::{entities::CardId, actions::StackActionOption};

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
