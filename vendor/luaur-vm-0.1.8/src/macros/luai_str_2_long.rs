#[macro_export]
#[allow(non_snake_case)]
macro_rules! luai_str2long {
    ($s:expr, $p:expr, $base:expr) => {
        unsafe { crate::macros::luai_str_2_long::strtoll($s, $p, $base) }
    };
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" {
    pub fn strtoll(
        s: *const core::ffi::c_char,
        endptr: *mut *mut core::ffi::c_char,
        base: core::ffi::c_int,
    ) -> i64;
}

#[cfg(target_arch = "wasm32")]
#[inline]
pub fn strtoll(
    _s: *const core::ffi::c_char,
    _endptr: *mut *mut core::ffi::c_char,
    _base: core::ffi::c_int,
) -> i64 {
    0
}

pub use luai_str2long;
