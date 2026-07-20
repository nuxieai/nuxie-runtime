#[inline]
pub fn safejson(ch: core::ffi::c_char) -> bool {
    (ch as u8) < 128
        && ch >= 32
        && ch != b'\\' as core::ffi::c_char
        && ch != b'\"' as core::ffi::c_char
}
