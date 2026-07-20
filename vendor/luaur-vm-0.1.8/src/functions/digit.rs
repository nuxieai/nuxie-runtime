pub(crate) unsafe fn digit(c: core::ffi::c_int) -> core::ffi::c_int {
    if ('0' as core::ffi::c_int) <= c && c <= ('9' as core::ffi::c_int) {
        1
    } else {
        0
    }
}
