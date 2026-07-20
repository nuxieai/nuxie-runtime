//! Node: `cxx:Function:Luau.VM:VM/src/linit.cpp:42:luaL_openlibs`
//! Source: `VM/src/linit.cpp:42-60` (hand-ported)

use crate::functions::lua_call::lua_call;
use crate::functions::lua_pushstring::lua_pushstring;
use crate::functions::luaopen_base::luaopen_base;
use crate::functions::luaopen_bit_32::luaopen_bit32;
use crate::functions::luaopen_buffer::luaopen_buffer;
use crate::functions::luaopen_class::luaopen_class;
use crate::functions::luaopen_coroutine::luaopen_coroutine;
use crate::functions::luaopen_debug::luaopen_debug;
use crate::functions::luaopen_integer::luaopen_integer;
use crate::functions::luaopen_math::luaopen_math;
use crate::functions::luaopen_os::luaopen_os;
use crate::functions::luaopen_string::luaopen_string;
use crate::functions::luaopen_table::luaopen_table;
use crate::functions::luaopen_utf_8::luaopen_utf_8;
use crate::functions::luaopen_vector::luaopen_vector;
use crate::macros::lua_pushcfunction::LUA_PUSHCFUNCTION;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_l_openlibs(l: *mut lua_State) {
    let lualibs = [
        LuaLReg {
            name: c"".as_ptr(),
            func: Some(luaopen_base),
        },
        LuaLReg {
            name: c"coroutine".as_ptr(),
            func: Some(luaopen_coroutine),
        },
        LuaLReg {
            name: c"table".as_ptr(),
            func: Some(luaopen_table),
        },
        LuaLReg {
            name: c"os".as_ptr(),
            func: Some(luaopen_os),
        },
        LuaLReg {
            name: c"string".as_ptr(),
            func: Some(luaopen_string),
        },
        LuaLReg {
            name: c"math".as_ptr(),
            func: Some(luaopen_math),
        },
        LuaLReg {
            name: c"debug".as_ptr(),
            func: Some(luaopen_debug),
        },
        LuaLReg {
            name: c"utf8".as_ptr(),
            func: Some(luaopen_utf_8),
        },
        LuaLReg {
            name: c"bit32".as_ptr(),
            func: Some(luaopen_bit32),
        },
        LuaLReg {
            name: c"buffer".as_ptr(),
            func: Some(luaopen_buffer),
        },
        LuaLReg {
            name: c"vector".as_ptr(),
            func: Some(luaopen_vector),
        },
        LuaLReg {
            name: c"int64".as_ptr(),
            func: Some(luaopen_integer),
        },
        LuaLReg {
            name: core::ptr::null(),
            func: None,
        },
    ];

    let lualibs_nointeger = [
        LuaLReg {
            name: c"".as_ptr(),
            func: Some(luaopen_base),
        },
        LuaLReg {
            name: c"coroutine".as_ptr(),
            func: Some(luaopen_coroutine),
        },
        LuaLReg {
            name: c"table".as_ptr(),
            func: Some(luaopen_table),
        },
        LuaLReg {
            name: c"os".as_ptr(),
            func: Some(luaopen_os),
        },
        LuaLReg {
            name: c"string".as_ptr(),
            func: Some(luaopen_string),
        },
        LuaLReg {
            name: c"math".as_ptr(),
            func: Some(luaopen_math),
        },
        LuaLReg {
            name: c"debug".as_ptr(),
            func: Some(luaopen_debug),
        },
        LuaLReg {
            name: c"utf8".as_ptr(),
            func: Some(luaopen_utf_8),
        },
        LuaLReg {
            name: c"bit32".as_ptr(),
            func: Some(luaopen_bit32),
        },
        LuaLReg {
            name: c"buffer".as_ptr(),
            func: Some(luaopen_buffer),
        },
        LuaLReg {
            name: c"vector".as_ptr(),
            func: Some(luaopen_vector),
        },
        LuaLReg {
            name: core::ptr::null(),
            func: None,
        },
    ];

    let libs = if luaur_common::FFlag::LuauIntegerLibrary.get() {
        lualibs.as_ptr()
    } else {
        lualibs_nointeger.as_ptr()
    };

    let mut lib = libs;
    while (*lib).func.is_some() {
        LUA_PUSHCFUNCTION(l, (*lib).func, core::ptr::null());
        lua_pushstring(l, (*lib).name);
        lua_call(l, 1, 0);
        lib = lib.add(1);
    }

    if luaur_common::FFlag::DebugLuauUserDefinedClassesRuntime.get() {
        LUA_PUSHCFUNCTION(l, Some(luaopen_class), core::ptr::null());
        lua_pushstring(l, c"class".as_ptr());
        lua_call(l, 1, 0);
    }
}
