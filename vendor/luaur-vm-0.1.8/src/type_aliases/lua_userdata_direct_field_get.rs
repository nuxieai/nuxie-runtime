#[allow(non_camel_case_types)]
pub type lua_UserdataDirectFieldGet =
    Option<unsafe extern "C" fn(ud: *mut core::ffi::c_void, result: *mut core::ffi::c_void)>;

#[allow(non_camel_case_types)]
pub type LuaUserdataDirectFieldGet = lua_UserdataDirectFieldGet;
