#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct AdvanceFlags(pub u16);

impl AdvanceFlags {
    pub const NONE: Self = Self(0);
    pub const ADVANCE_NESTED: Self = Self(1 << 0);
    pub const ANIMATE: Self = Self(1 << 1);
    pub const IS_ROOT: Self = Self(1 << 2);
    pub const NEW_FRAME: Self = Self(1 << 3);

    pub const fn contains(self, mask: Self) -> bool {
        self.0 & mask.0 == mask.0
    }
}

impl std::ops::BitAnd for AdvanceFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for AdvanceFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitXor for AdvanceFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for AdvanceFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl std::ops::BitAndAssign for AdvanceFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOrAssign for AdvanceFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXorAssign for AdvanceFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
