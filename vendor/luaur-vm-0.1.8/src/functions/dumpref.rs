use crate::records::gc_object::GCObject;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpref(f: *mut core::ffi::c_void, o: *mut GCObject) {
    let fmt = "\"%p\"\0";

    extern "C" {
        fn fprintf(
            stream: *mut core::ffi::c_void,
            format: *const core::ffi::c_char,
            ...
        ) -> core::ffi::c_int;
    }

    fprintf(f, fmt.as_ptr() as *const core::ffi::c_char, o);
}
