#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TKey {
    pub(crate) value: crate::type_aliases::value::Value,
    pub(crate) extra: [core::ffi::c_int; 1],
    /// C++ bitfields `unsigned tt : 4; int next : 28;` packed into one 4-byte
    /// word (low 4 bits = tt, high 28 bits = signed next). Keeping them as two
    /// separate `c_int`s made TKey 24 bytes (LuaNode 40) instead of 16/32, which
    /// broke the JIT ABI guard. Access via tt()/next()/set_tt()/set_next() only.
    pub(crate) tt_next: core::ffi::c_uint,
}

#[allow(non_camel_case_types)]
pub type t_key = TKey;

impl TKey {
    #[inline]
    pub fn tt(&self) -> core::ffi::c_int {
        (self.tt_next & 0xF) as core::ffi::c_int
    }

    #[inline]
    pub fn set_tt(&mut self, tt: core::ffi::c_int) {
        self.tt_next = (self.tt_next & !0xF) | ((tt as core::ffi::c_uint) & 0xF);
    }

    #[inline]
    pub fn next(&self) -> i32 {
        // `int next : 28` — sign-extend from the 28-bit field (bits 4..31).
        (self.tt_next as i32) >> 4
    }

    #[inline]
    pub fn set_next(&mut self, next: i32) {
        self.tt_next = (self.tt_next & 0xF) | ((next as u32) << 4);
    }
}

impl Default for TKey {
    fn default() -> Self {
        Self {
            value: crate::type_aliases::value::Value::default(),
            extra: [0; 1],
            tt_next: 0,
        }
    }
}

impl core::fmt::Debug for TKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TKey")
            .field("extra", &self.extra)
            .field("tt", &self.tt())
            .field("next", &self.next())
            .finish_non_exhaustive()
    }
}
