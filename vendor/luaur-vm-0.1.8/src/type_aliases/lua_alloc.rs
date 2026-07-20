#[allow(non_camel_case_types)]
pub type lua_Alloc = Option<
    unsafe extern "C" fn(
        ud: *mut core::ffi::c_void,
        ptr: *mut core::ffi::c_void,
        osize: usize,
        nsize: usize,
    ) -> *mut core::ffi::c_void,
>;

#[allow(non_camel_case_types)]
pub type LuaAlloc = lua_Alloc;
