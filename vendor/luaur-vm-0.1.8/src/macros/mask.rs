use crate::macros::allones::ALLONES;

#[allow(non_snake_case)]
pub const fn mask(n: core::ffi::c_int) -> core::ffi::c_uint {
    !((ALLONES << 1).wrapping_shl((n - 1) as u32))
}
