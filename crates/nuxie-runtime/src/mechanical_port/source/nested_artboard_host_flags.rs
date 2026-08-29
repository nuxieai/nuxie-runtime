#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NestedArtboardHostFlags(pub u8);

impl NestedArtboardHostFlags {
    pub const NONE: Self = Self(0);
    pub const PENDING_STATEFUL_BINDING: Self = Self(1 << 0);
    pub const ARTBOARD_DATA_BOUND: Self = Self(1 << 1);
}
