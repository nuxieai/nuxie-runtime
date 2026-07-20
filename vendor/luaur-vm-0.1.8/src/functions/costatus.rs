//! Node: `cxx:Function:Luau.VM:VM/src/lcorolib.cpp:14:costatus`
//!
//! `coroutine.status` — push the textual status of the thread argument. The
//! index order matches `lua_costatus`: 0 running, 1 suspended, 2 normal, 3/4
//! dead (the C++ `statnames` table repeats "dead" for COERR/COFIN).

use crate::functions::lua_costatus::lua_costatus;
use crate::functions::lua_pushstring::lua_pushstring;
use crate::functions::lua_tothread::lua_tothread;
use crate::macros::lua_l_argexpected::luaL_argexpected;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};

pub fn costatus(l: *mut lua_State) -> c_int {
    unsafe {
        let co = lua_tothread(l, 1);
        luaL_argexpected!(l, !co.is_null(), 1, "thread");

        let name: *const c_char = match lua_costatus(l, co) {
            0 => c"running".as_ptr(),
            1 => c"suspended".as_ptr(),
            2 => c"normal".as_ptr(),
            _ => c"dead".as_ptr(),
        };
        lua_pushstring(l, name);
        1
    }
}
