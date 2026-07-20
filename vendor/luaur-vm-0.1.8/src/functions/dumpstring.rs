use crate::functions::dumpstringdata::dumpstringdata;
use crate::macros::sizestring::sizestring;
use crate::type_aliases::t_string::TString;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpstring(f: *mut core::ffi::c_void, ts: *mut TString) {
    let ts = &*ts;
    let fmt = "{\"type\":\"string\",\"cat\":%d,\"size\":%d,\"data\":\"";

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
        ts.hdr.memcat as core::ffi::c_int,
        sizestring(ts.len as usize) as core::ffi::c_int,
    );

    dumpstringdata(f, ts.data.as_ptr(), ts.len as usize);

    fprintf(f, b"\"}\"".as_ptr() as *const core::ffi::c_char);
}
