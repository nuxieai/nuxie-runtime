use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::records::lua_exception::LuaResult;

#[allow(non_upper_case_globals)]
pub const LUA_PUSHLITERAL: unsafe fn(
    *mut core::ffi::c_void,
    *const core::ffi::c_char,
) -> LuaResult<()> = |l, s| unsafe {
    // In C++, lua_pushliteral(L, s) uses (sizeof(s) / sizeof(char)) - 1.
    // Since this is a macro-like constant function in Rust, we expect s to be a pointer to a null-terminated string.
    // We use the length of the string (excluding null terminator) to match the C++ behavior.
    let len = core::ffi::CStr::from_ptr(s).to_bytes().len();

    lua_pushlstring(l.cast(), s, len)
};

#[allow(non_upper_case_globals)]
pub const lua_pushliteral: unsafe fn(
    *mut core::ffi::c_void,
    *const core::ffi::c_char,
) -> LuaResult<()> = LUA_PUSHLITERAL;
