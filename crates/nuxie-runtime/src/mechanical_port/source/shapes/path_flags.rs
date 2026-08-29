use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct PathFlags(pub u8);
impl PathFlags {
    pub const NONE: Self = Self(0);
    pub const LOCAL: Self = Self(1 << 1);
    pub const WORLD: Self = Self(1 << 2);
    pub const CLIPPING: Self = Self(1 << 3);
    pub const FOLLOW_PATH: Self = Self(1 << 4);
    pub const NEVER_DEFER_UPDATE: Self = Self(1 << 5);
    pub const LOCAL_CLOCKWISE: Self = Self(1 << 6);
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl BitAnd for PathFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitXor for PathFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl BitOr for PathFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Not for PathFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
impl BitOrAssign for PathFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl BitAndAssign for PathFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl BitXorAssign for PathFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
