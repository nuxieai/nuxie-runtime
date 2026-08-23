//! Mechanical declaration translation of
//! `renderer/include/rive/renderer/gl/load_store_actions_ext.hpp`.

#![allow(non_snake_case, non_upper_case_globals)]

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_load_store_actions_ext.hpp");

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LoadStoreActionsEXT(pub(crate) u32);

impl LoadStoreActionsEXT {
    pub(crate) const none: Self = Self(0);
    pub(crate) const clearColor: Self = Self(1 << 0);
    pub(crate) const loadColor: Self = Self(1 << 1);
    pub(crate) const storeColor: Self = Self(1 << 2);
    pub(crate) const clearCoverage: Self = Self(1 << 3);
    pub(crate) const clearClip: Self = Self(1 << 4);

    pub(crate) const fn has(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl core::ops::BitOr for LoadStoreActionsEXT {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for LoadStoreActionsEXT {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub(crate) const LOAD_STORE_ACTIONS_EXT_COUNT: u32 = 5;
