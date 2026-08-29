#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawableFlag(pub u16);

impl DrawableFlag {
    pub const NONE: Self = Self(0);
    pub const HIDDEN: Self = Self(1 << 0);
    pub const LOCKED: Self = Self(1 << 1);
    pub const DISCONNECTED: Self = Self(1 << 2);
    pub const OPAQUE: Self = Self(1 << 3);
    pub const WORLD_BOUNDS_CLEAN: Self = Self(1 << 4);
    pub const PARTICIPATES_IN_LAYOUT: Self = Self(1 << 8);
}
