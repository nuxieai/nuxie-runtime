#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
pub struct lua_Page {
    pub(crate) prev: *mut lua_Page,
    pub(crate) next: *mut lua_Page,
    pub(crate) listprev: *mut lua_Page,
    pub(crate) listnext: *mut lua_Page,
    pub(crate) pageSize: core::ffi::c_int,
    pub(crate) blockSize: core::ffi::c_int,
    pub(crate) freeList: *mut core::ffi::c_void,
    pub(crate) freeNext: core::ffi::c_int,
    pub(crate) busyBlocks: core::ffi::c_int,
    #[cfg(target_pointer_width = "64")]
    pub(crate) padding: [core::ffi::c_char; 8],
    #[cfg(not(target_pointer_width = "64"))]
    pub(crate) padding: [core::ffi::c_char; 12],
    pub(crate) data: [core::ffi::c_char; 1],
}
