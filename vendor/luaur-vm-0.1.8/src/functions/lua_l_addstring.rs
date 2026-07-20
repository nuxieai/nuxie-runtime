//! `luaL_addstring` (VM/include/lualib.h macro) — append a NUL-terminated C
//! string to a `luaL_Strbuf` via `luaL_addlstring` with `strlen(s)`.

use crate::functions::lua_l_addlstring::lua_l_addlstring;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use core::ffi::c_char;

pub unsafe fn lua_l_addstring(b: *mut LuaLStrbuf, s: *const c_char) {
    let len = core::ffi::CStr::from_ptr(s).to_bytes().len();
    lua_l_addlstring(b, s, len);
}
