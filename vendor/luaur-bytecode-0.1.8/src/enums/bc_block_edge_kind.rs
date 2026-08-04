#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcBlockEdgeKind {
    Branch,
    Fallthrough,
    Loop,
}
