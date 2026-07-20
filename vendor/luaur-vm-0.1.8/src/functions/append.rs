#[allow(non_snake_case)]
pub fn append(
    buf: *mut core::ffi::c_char,
    bufsize: usize,
    offset: usize,
    data: *const core::ffi::c_char,
) -> usize {
    let size = unsafe {
        core::ffi::CStr::from_ptr(data as *mut core::ffi::c_char)
            .to_bytes()
            .len()
    };
    let copy = if offset + size >= bufsize {
        bufsize - offset - 1
    } else {
        size
    };

    let dst = unsafe { core::slice::from_raw_parts_mut(buf.add(offset) as *mut u8, copy) };
    let src = unsafe { core::slice::from_raw_parts(data as *const u8, copy) };
    dst.copy_from_slice(src);

    offset + copy
}
