#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct lua_TValue {
    pub value: crate::type_aliases::value::Value,
    pub extra: [core::ffi::c_int; 1],
    pub tt: core::ffi::c_int,
}

#[allow(non_camel_case_types)]
pub type TValue = lua_TValue;

impl Default for lua_TValue {
    fn default() -> Self {
        Self {
            value: crate::type_aliases::value::Value::default(),
            extra: [0; 1],
            tt: 0,
        }
    }
}

impl core::fmt::Debug for lua_TValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("lua_TValue")
            .field("extra", &self.extra)
            .field("tt", &self.tt)
            .finish_non_exhaustive()
    }
}

impl lua_TValue {
    /// Tag accessor mirroring `TKey::tt()` so the C++ duck-typed tag macros
    /// (`ttype!`, `setttype!`, `iscollectable!`) work on values AND keys.
    #[inline]
    pub fn tt(&self) -> core::ffi::c_int {
        self.tt
    }

    #[inline]
    pub fn set_tt(&mut self, tt: core::ffi::c_int) {
        self.tt = tt;
    }
}
