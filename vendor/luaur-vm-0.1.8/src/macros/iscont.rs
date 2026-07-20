#[allow(non_snake_case)]
#[inline]
pub unsafe fn iscont(p: *const core::ffi::c_char) -> bool {
    ((*p as u8) & 0xC0) == 0x80
}
