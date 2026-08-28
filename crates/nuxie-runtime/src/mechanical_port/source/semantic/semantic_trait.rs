#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct SemanticTrait(pub u32);

impl SemanticTrait {
    pub const NONE: Self = Self(0);
    pub const EXPANDABLE: Self = Self(1 << 0);
    pub const SELECTABLE: Self = Self(1 << 1);
    pub const CHECKABLE: Self = Self(1 << 2);
    pub const TOGGLEABLE: Self = Self(1 << 3);
    pub const REQUIRABLE: Self = Self(1 << 4);
    pub const ENABLABLE: Self = Self(1 << 5);
    pub const FOCUSABLE: Self = Self(1 << 6);
}

impl core::ops::BitOr for SemanticTrait {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for SemanticTrait {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitOrAssign for SemanticTrait {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub fn has_semantic_trait(flags: u32, value: SemanticTrait) -> bool {
    flags & value.0 != 0
}
