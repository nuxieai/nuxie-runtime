#[allow(non_snake_case)]
pub unsafe fn printspecial(
    buf: *mut core::ffi::c_char,
    sign: core::ffi::c_int,
    fraction: u64,
) -> *mut core::ffi::c_char {
    if fraction == 0 {
        let src = b"-inf\0";
        let offset = (1 - sign) as usize;
        core::ptr::copy_nonoverlapping(src.as_ptr().add(offset), buf as *mut u8, 4);
        buf.add((3 + sign) as usize)
    } else {
        let src = b"nan\0";
        core::ptr::copy_nonoverlapping(src.as_ptr(), buf as *mut u8, 4);
        buf.add(3)
    }
}
