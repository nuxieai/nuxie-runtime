#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct SemanticState(pub u32);

impl SemanticState {
    pub const NONE: Self = Self(0);
    pub const EXPANDED: Self = Self(1 << 0);
    pub const SELECTED: Self = Self(1 << 1);
    pub const TOGGLED: Self = Self(1 << 4);
    pub const REQUIRED: Self = Self(1 << 5);
    pub const DISABLED: Self = Self(1 << 6);
    pub const FOCUSED: Self = Self(1 << 7);
    pub const HIDDEN: Self = Self(1 << 8);
    pub const LIVE_REGION: Self = Self(1 << 9);
    pub const READ_ONLY: Self = Self(1 << 10);
    pub const MODAL: Self = Self(1 << 11);
    pub const OBSCURED: Self = Self(1 << 12);
    pub const MULTILINE: Self = Self(1 << 13);
}

impl core::ops::BitOr for SemanticState {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for SemanticState {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitOrAssign for SemanticState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub fn has_semantic_state(flags: u32, flag: SemanticState) -> bool {
    flags & flag.0 != 0
}

/// A checkable node's two-bit field, not an independent semantic flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SemanticCheckState {
    Unchecked = 0,
    Checked = 1,
    Mixed = 2,
}

/// The unused fourth field value is read as Mixed.
pub const fn check_state_of(flags: u32) -> SemanticCheckState {
    use crate::mechanical_port::source::generated::semantic::semantic_data_base::SemanticDataBase;
    let value = (flags & SemanticDataBase::IS_CHECKED_FIELD_MASK)
        >> SemanticDataBase::IS_CHECKED_BIT_OFFSET;
    match value {
        0 => SemanticCheckState::Unchecked,
        1 => SemanticCheckState::Checked,
        _ => SemanticCheckState::Mixed,
    }
}
