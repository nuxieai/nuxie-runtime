#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionState {
    AlwaysFalse,
    AlwaysTrue,
    Unknown,
}
