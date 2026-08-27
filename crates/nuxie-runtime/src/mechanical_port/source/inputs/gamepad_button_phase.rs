pub struct GamepadButtonPhaseMask;

impl GamepadButtonPhaseMask {
    pub const DOWN: u32 = 1 << 0;
    pub const UP: u32 = 1 << 1;
    pub const ALL: u32 = Self::DOWN | Self::UP;
}
