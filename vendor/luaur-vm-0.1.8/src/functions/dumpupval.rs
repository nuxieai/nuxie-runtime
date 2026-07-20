use crate::functions::dumpref::dumpref;
use crate::macros::gcvalue::gcvalue;
use crate::macros::iscollectable::iscollectable;
use crate::macros::upisopen::upisopen;
use crate::type_aliases::up_val::UpVal;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpupval(f: *mut core::ffi::c_void, uv: *mut UpVal) {
    extern "C" {
        fn fprintf(
            stream: *mut core::ffi::c_void,
            format: *const core::ffi::c_char,
            ...
        ) -> core::ffi::c_int;
    }

    let uv_ref = &*uv;

    let is_open = upisopen!(uv);

    fprintf(
        f,
        b"{\"type\":\"upvalue\",\"cat\":%d,\"size\":%d,\"open\":%s\0".as_ptr()
            as *const core::ffi::c_char,
        uv_ref.hdr.memcat as core::ffi::c_int,
        core::mem::size_of::<UpVal>() as core::ffi::c_int,
        if is_open {
            b"true\0".as_ptr()
        } else {
            b"false\0".as_ptr()
        } as *const core::ffi::c_char,
    );

    if iscollectable!(uv_ref.v) {
        fprintf(f, b",\"object\":\0".as_ptr() as *const core::ffi::c_char);
        dumpref(f, gcvalue!(uv_ref.v));
    }

    fprintf(f, b"}\0".as_ptr() as *const core::ffi::c_char);
}
