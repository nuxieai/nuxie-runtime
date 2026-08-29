use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StateTransitionFlags(pub u8);

impl StateTransitionFlags {
    pub const NONE: Self = Self(0);
    pub const DISABLED: Self = Self(1 << 0);
    pub const DURATION_IS_PERCENTAGE: Self = Self(1 << 1);
    pub const ENABLE_EXIT_TIME: Self = Self(1 << 2);
    pub const EXIT_TIME_IS_PERCENTAGE: Self = Self(1 << 3);
    pub const PAUSE_ON_EXIT: Self = Self(1 << 4);
    pub const ENABLE_EARLY_EXIT: Self = Self(1 << 5);
}

impl BitAnd for StateTransitionFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for StateTransitionFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitXor for StateTransitionFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl Not for StateTransitionFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
impl BitAndAssign for StateTransitionFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl BitOrAssign for StateTransitionFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl BitXorAssign for StateTransitionFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
