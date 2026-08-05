//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:71:luaL_where`
//! Source: `VM/src/laux.cpp:71-83` (hand-ported)

use core::ffi::c_int;

use crate::functions::lua_getinfo::lua_getinfo;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::records::lua_debug::LuaDebug;
use crate::type_aliases::lua_state::lua_State;

unsafe fn push_where(L: *mut lua_State, source: *const core::ffi::c_char, line: c_int) {
    let source = core::ffi::CStr::from_ptr(source).to_bytes();
    let line = alloc::format!("{line}");
    let mut result = alloc::vec::Vec::with_capacity(source.len() + line.len() + 3);
    result.extend_from_slice(source);
    result.push(b':');
    result.extend_from_slice(line.as_bytes());
    result.extend_from_slice(b": ");
    lua_pushlstring(L, result.as_ptr().cast(), result.len());
}

#[allow(non_snake_case)]
pub unsafe fn lua_l_where(L: *mut lua_State, level: c_int) {
    let mut ar: LuaDebug = core::mem::zeroed();
    if lua_getinfo(L, level, c"sl".as_ptr(), &mut ar) != 0 && ar.currentline > 0 {
        push_where(L, ar.source, ar.currentline);
        return;
    }

    if lua_getinfo(L, 0, c"sl".as_ptr(), &mut ar) != 0 && ar.currentline > 0 {
        push_where(L, ar.source, ar.currentline);
        return;
    }

    lua_rawcheckstack(L, 1);
    lua_pushlstring(L, c":: ".as_ptr(), 3);
}
