#[derive(Debug, Clone, Copy)]
pub enum MTGError {
    CostNotPaid,
    CantCast,
    CastNonExistentSpell,
    PlayerDoesntExist,
    TargetNotChosen,
}
