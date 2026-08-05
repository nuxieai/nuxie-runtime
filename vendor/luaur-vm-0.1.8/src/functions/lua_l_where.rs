//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:71:luaL_where`
//! Source: `VM/src/laux.cpp:71-83` (hand-ported)

use core::ffi::c_int;

use crate::functions::lua_getinfo::lua_getinfo;
use crate::functions::lua_pushfstring_l::lua_pushfstring_l;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::records::lua_debug::LuaDebug;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_where(L: *mut lua_State, level: c_int) {
    let mut ar: LuaDebug = core::mem::zeroed();
    if lua_getinfo(L, level, c"sl".as_ptr(), &mut ar) != 0 && ar.currentline > 0 {
        let source = core::ffi::CStr::from_ptr(ar.source).to_string_lossy();
        lua_pushfstring_l(
            L,
            c"%s:%d: ".as_ptr(),
            format_args!("{}:{}: ", source, ar.currentline),
        );
        return;
    }

    if lua_getinfo(L, 0, c"sl".as_ptr(), &mut ar) != 0 && ar.currentline > 0 {
        let source = core::ffi::CStr::from_ptr(ar.source).to_string_lossy();
        lua_pushfstring_l(
            L,
            c"%s:%d: ".as_ptr(),
            format_args!("{}:{}: ", source, ar.currentline),
        );
        return;
    }

    lua_rawcheckstack(L, 1);
    lua_pushlstring(L, c":: ".as_ptr(), 3);
}
