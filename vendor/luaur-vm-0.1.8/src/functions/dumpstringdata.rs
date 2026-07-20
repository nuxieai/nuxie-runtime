use crate::functions::safejson::safejson;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpstringdata(
    f: *mut core::ffi::c_void,
    data: *const core::ffi::c_char,
    len: usize,
) {
    let slice = core::slice::from_raw_parts(data, len);
    for &ch in slice {
        let out = if safejson(ch) {
            ch
        } else {
            '?' as core::ffi::c_char
        };

        // Note: fputc is provided by the system's C library.
        // In this crate's context, we call it via the extern "C" linkage.
        extern "C" {
            fn fputc(c: core::ffi::c_int, stream: *mut core::ffi::c_void) -> core::ffi::c_int;
        }

        fputc(out as core::ffi::c_int, f);
    }
}
