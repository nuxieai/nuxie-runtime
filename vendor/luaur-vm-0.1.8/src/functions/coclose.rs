//! Node: `cxx:Function:Luau.VM:VM/src/lcorolib.cpp:219:coclose`
//!
//! `coroutine.close` — close a dead/suspended thread: error if it is running or
//! normal, otherwise push `true` (and reset) for a clean thread, or `false` plus
//! the error object for an errored one, then reset it.

use crate::enums::lua_co_status::lua_CoStatus;
use crate::enums::lua_status::lua_Status;
use crate::functions::lua_costatus::lua_costatus;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::functions::lua_pushstring::lua_pushstring;
use crate::functions::lua_resetthread::lua_resetthread;
use crate::functions::lua_tothread::lua_tothread;
use crate::functions::lua_xmove::lua_xmove;
use crate::macros::lua_l_argexpected::luaL_argexpected;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub fn coclose(l: *mut lua_State) -> c_int {
    unsafe {
        let co = lua_tothread(l, 1);
        luaL_argexpected!(l, !co.is_null(), 1, "thread");

        let status = lua_costatus(l, co);
        if status != lua_CoStatus::LUA_COFIN as c_int
            && status != lua_CoStatus::LUA_COERR as c_int
            && status != lua_CoStatus::LUA_COSUS as c_int
        {
            let sname = match status {
                0 => "running",
                1 => "suspended",
                2 => "normal",
                _ => "dead",
            };
            luaL_error!(l, "cannot close {} coroutine", sname);
        }

        if (*co).status as c_int == lua_Status::LUA_OK as c_int
            || (*co).status as c_int == lua_Status::LUA_YIELD as c_int
        {
            lua_pushboolean(l, 1);
            lua_resetthread(co);
            1
        } else {
            lua_pushboolean(l, 0);

            if (*co).status as c_int == lua_Status::LUA_ERRMEM as c_int {
                lua_pushstring(l, c"not enough memory".as_ptr());
            } else if (*co).status as c_int == lua_Status::LUA_ERRERR as c_int {
                lua_pushstring(l, c"error in error handling".as_ptr());
            } else if lua_gettop(co) != 0 {
                lua_xmove(co, l, 1); // move error message
            }

            lua_resetthread(co);
            2
        }
    }
}
