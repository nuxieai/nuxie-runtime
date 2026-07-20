#[allow(non_snake_case)]
#[inline(always)]
pub const fn uchar(c: core::ffi::c_int) -> core::ffi::c_uchar {
    c as core::ffi::c_uchar
}
