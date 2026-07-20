use crate::macros::sizebuffer::sizebuffer;
use crate::type_aliases::buffer::Buffer;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpbuffer(f: *mut core::ffi::c_void, b: *mut Buffer) {
    let b = &*b;
    let fmt = "{\"type\":\"buffer\",\"cat\":%d,\"size\":%d}\0";

    extern "C" {
        fn fprintf(
            stream: *mut core::ffi::c_void,
            format: *const core::ffi::c_char,
            ...
        ) -> core::ffi::c_int;
    }

    fprintf(
        f,
        fmt.as_ptr() as *const core::ffi::c_char,
        b.memcat as core::ffi::c_int,
        sizebuffer(b.len as usize) as core::ffi::c_int,
    );
}
