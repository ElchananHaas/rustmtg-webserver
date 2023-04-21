use crate::entities::CardId;

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
}
