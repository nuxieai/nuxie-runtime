#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct BinaryOpPriority {
    pub(crate) left: u8,
    pub(crate) right: u8,
}
