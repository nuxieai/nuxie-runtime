//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:304:luaL_register`
//! Source: `VM/src/laux.cpp:304-327` (hand-ported)

use core::ffi::c_char;

use crate::functions::libsize::libsize;
use crate::functions::lua_getfield::lua_getfield;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::functions::lua_l_findtable::luaL_findtable;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_remove::lua_remove;
use crate::functions::lua_setfield::lua_setfield;
use crate::functions::lua_type::lua_type;
use crate::macros::lua_globalsindex::LUA_GLOBALSINDEX;
use crate::macros::lua_pop::lua_pop;
use crate::macros::lua_pushcfunction::LUA_PUSHCFUNCTION;
use crate::macros::lua_registryindex::LUA_REGISTRYINDEX;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_register(L: *mut lua_State, libname: *const c_char, mut l: *const LuaLReg) {
    if !libname.is_null() {
        let size = libsize(l);
        luaL_findtable(L, LUA_REGISTRYINDEX, c"_LOADED".as_ptr(), 1);
        lua_getfield(L, -1, libname);
        if lua_type(L, -1) != crate::enums::lua_type::lua_Type::LUA_TTABLE as i32 {
            lua_pop(L, 1);
            if !luaL_findtable(L, LUA_GLOBALSINDEX, libname, size).is_null() {
                let name = core::ffi::CStr::from_ptr(libname).to_string_lossy();
                lua_l_error_l(
                    L,
                    c"name conflict for module '%s'".as_ptr(),
                    format_args!("name conflict for module '{}'", name),
                );
            }
            lua_pushvalue(L, -1);
            lua_setfield(L, -3, libname);
        }
        lua_remove(L, -2);
    }

    while !(*l).name.is_null() {
        LUA_PUSHCFUNCTION(L, (*l).func, (*l).name);
        lua_setfield(L, -2, (*l).name);
        l = l.add(1);
    }
}
