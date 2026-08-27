#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoystickFlags(pub u8);

impl JoystickFlags {
    pub const NONE: Self = Self(0);
    pub const INVERT_X: Self = Self(1 << 0);
    pub const INVERT_Y: Self = Self(1 << 1);
    pub const WORLD_SPACE: Self = Self(1 << 2);
}
