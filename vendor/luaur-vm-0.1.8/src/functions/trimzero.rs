#[inline]
pub unsafe fn trimzero(mut end: *mut core::ffi::c_char) -> *mut core::ffi::c_char {
    while *end.offset(-1) == b'0' as core::ffi::c_char {
        end = end.offset(-1);
    }

    end
}
