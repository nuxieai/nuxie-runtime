#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvanceFlags(pub u16);

impl AdvanceFlags {
    pub const NONE: Self = Self(0);
    pub const ADVANCE_NESTED: Self = Self(1 << 0);
    pub const ANIMATE: Self = Self(1 << 1);
    pub const IS_ROOT: Self = Self(1 << 2);
    pub const NEW_FRAME: Self = Self(1 << 3);
}
