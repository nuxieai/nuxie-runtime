use std::ops::BitAnd;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LayerStateFlags(pub u8);

impl LayerStateFlags {
    pub const NONE: Self = Self(0);
    pub const RANDOM: Self = Self(1 << 0);
    pub const RESET: Self = Self(1 << 1);
}

impl BitAnd for LayerStateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
