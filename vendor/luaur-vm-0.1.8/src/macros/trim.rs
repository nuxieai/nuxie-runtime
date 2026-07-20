#[allow(non_snake_case)]
#[inline]
pub const fn trim(x: core::ffi::c_uint) -> core::ffi::c_uint {
    x & crate::macros::allones::ALLONES
}
