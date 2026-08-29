#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataBindFlags(pub u16);

impl DataBindFlags {
    pub const NONE: Self = Self(0);
    pub const DIRECTION: Self = Self(1 << 0);
    pub const TWO_WAY: Self = Self(1 << 1);
    pub const ONCE: Self = Self(1 << 2);
    pub const SOURCE_TO_TARGET_RUNS_FIRST: Self = Self(1 << 3);
    pub const NAME_BASED: Self = Self(1 << 4);
    pub const TO_TARGET: Self = Self(0);
    pub const TO_SOURCE: Self = Self(1 << 0);
}
