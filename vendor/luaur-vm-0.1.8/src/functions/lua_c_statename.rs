pub(crate) const GCSpause: core::ffi::c_int = 0;
pub(crate) const GCSpropagate: core::ffi::c_int = 1;
pub(crate) const GCSpropagateagain: core::ffi::c_int = 2;
pub(crate) const GCSatomic: core::ffi::c_int = 3;
pub(crate) const GCSsweep: core::ffi::c_int = 4;

#[allow(non_snake_case)]
pub fn luaC_statename(state: core::ffi::c_int) -> *const core::ffi::c_char {
    match state {
        GCSpause => c"pause".as_ptr(),
        GCSpropagate => c"mark".as_ptr(),
        GCSpropagateagain => c"remark".as_ptr(),
        GCSatomic => c"atomic".as_ptr(),
        GCSsweep => c"sweep".as_ptr(),
        _ => core::ptr::null(),
    }
}
