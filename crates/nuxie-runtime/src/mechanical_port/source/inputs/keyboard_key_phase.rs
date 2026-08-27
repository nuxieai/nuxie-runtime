pub struct KeyboardKeyPhaseMask;

impl KeyboardKeyPhaseMask {
    pub const DOWN: u32 = 1 << 0;
    pub const REPEAT: u32 = 1 << 1;
    pub const UP: u32 = 1 << 2;
    pub const ALL: u32 = Self::DOWN | Self::REPEAT | Self::UP;
}
