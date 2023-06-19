use common::{entities::CardId, actions::StackActionOption};
use mtg_log_macro::MTGLoggable;
use common::log::{MTGLog, GameContext};
#[derive(Debug, Clone, MTGLoggable)]
pub enum Entry{
    DiesFromZeroOrLessToughness(CardId),
    DestroyFromDamage(CardId),
    DetachedEnchantmentDies(CardId),
    EnchantmentFallsOff(CardId),
    CastFailedFromRestriction(CardId),
    ManaCostNotPaid(CardId),
    Cast(StackActionOption)
}