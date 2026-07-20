use crate::functions::getthread::getthread;
use crate::functions::lua_l_optinteger::lua_l_optinteger;
use crate::functions::lua_l_traceback::lua_l_traceback;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_optstring::LUA_L_OPTSTRING;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};

pub fn db_traceback(l: *mut lua_State) -> c_int {
    unsafe {
        let mut arg: c_int = 0;
        let l1 = getthread(l, &mut arg);

        // luaL_optstring(L, arg + 1, NULL)
        // The macro LUA_L_OPTSTRING is a placeholder for the logic:
        // (luaL_optlstring(L, (n), (d), NULL))
        // We call the underlying function directly as per the foundation rules.
        let msg_ptr = crate::functions::lua_l_optlstring::lua_l_optlstring(
            l,
            arg + 1,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        let msg = if msg_ptr.is_null() {
            None
        } else {
            Some(core::ffi::CStr::from_ptr(msg_ptr).to_str().unwrap_or(""))
        };

        let default_level = if l == l1 { 1 } else { 0 };
        let level = lua_l_optinteger(l, arg + 2, default_level);

        luaL_argcheck!(l, level >= 0, arg + 2, "level can't be negative");

        lua_l_traceback(l, l1, msg, level);

        1
    }
}
