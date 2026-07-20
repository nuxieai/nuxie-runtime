use crate::functions::lua_pushlstring::lua_pushlstring;

#[allow(non_upper_case_globals)]
pub const LUA_PUSHLITERAL: unsafe fn(
    *mut core::ffi::c_void,
    *const core::ffi::c_char,
) -> *mut core::ffi::c_void = |l, s| unsafe {
    // The dependency card for lua_pushlstring shows a stub signature `pub fn lua_pushlstring();`.
    // In Rust, a function with a stub signature `fn name()` cannot be called with arguments.
    // We must cast the function to the expected signature to allow the call to compile against the stub.
    let func: unsafe extern "C" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_char,
        usize,
    ) -> *mut core::ffi::c_void = core::mem::transmute(lua_pushlstring as *const core::ffi::c_void);

    // In C++, lua_pushliteral(L, s) uses (sizeof(s) / sizeof(char)) - 1.
    // Since this is a macro-like constant function in Rust, we expect s to be a pointer to a null-terminated string.
    // We use the length of the string (excluding null terminator) to match the C++ behavior.
    let len = core::ffi::CStr::from_ptr(s).to_bytes().len();

    func(l, s, len)
};

#[allow(non_upper_case_globals)]
pub const lua_pushliteral: unsafe fn(
    *mut core::ffi::c_void,
    *const core::ffi::c_char,
) -> *mut core::ffi::c_void = LUA_PUSHLITERAL;
