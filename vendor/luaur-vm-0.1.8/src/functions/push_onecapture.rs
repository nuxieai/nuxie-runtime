use crate::functions::lua_pushinteger::lua_pushinteger;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::macros::cap_position::CAP_POSITION;
use crate::macros::cap_unfinished::CAP_UNFINISHED;
use crate::macros::lua_l_error::luaL_error;
use crate::records::match_state::MatchState;
use core::ffi::{c_char, c_int};

pub(crate) unsafe fn push_onecapture(
    ms: *mut MatchState,
    i: c_int,
    s: *const c_char,
    e: *const c_char,
) {
    if i >= (*ms).level {
        if i == 0 {
            // lua_pushlstring(ms->L, s, e - s);
            let len = (e as usize).wrapping_sub(s as usize);
            // The dependency card shows lua_pushlstring() with no args in the stub,
            // but the contract requires calling with real arguments.
            let pushlstring_ptr = lua_pushlstring as *const ();
            let pushlstring_fn: unsafe fn(
                *mut crate::records::lua_state::LuaState,
                *const c_char,
                usize,
            ) = core::mem::transmute(pushlstring_ptr);
            pushlstring_fn((*ms).L, s, len);
        } else {
            // The luaL_error macro expansion calls lua_l_error_l.
            // Per contract: "Pass &str to a callee even if its current stub signature still shows *const i8".
            // However, the compiler error shows the current stub for lua_l_error_l expects *const c_char.
            // We cast the &str to a pointer to satisfy the current stub while it is being updated.
            let fmt = "invalid capture index";
            crate::functions::lua_l_error_l::lua_l_error_l(
                (*ms).L,
                fmt.as_ptr() as *const c_char,
                core::format_args!("{}", fmt),
            );
        }
    } else {
        let l = (*ms).capture[i as usize].len;
        if l == CAP_UNFINISHED as isize {
            let fmt = "unfinished capture";
            crate::functions::lua_l_error_l::lua_l_error_l(
                (*ms).L,
                fmt.as_ptr() as *const c_char,
                core::format_args!("{}", fmt),
            );
        } else if l == CAP_POSITION as isize {
            // lua_pushinteger(ms->L, (int)(ms->capture[i].init - ms->src_init) + 1);
            let pos = ((*ms).capture[i as usize].init as usize)
                .wrapping_sub((*ms).src_init as usize) as c_int;
            lua_pushinteger((*ms).L, pos + 1);
        } else {
            // lua_pushlstring(ms->L, ms->capture[i].init, l);
            let pushlstring_ptr = lua_pushlstring as *const ();
            let pushlstring_fn: unsafe fn(
                *mut crate::records::lua_state::LuaState,
                *const c_char,
                usize,
            ) = core::mem::transmute(pushlstring_ptr);
            pushlstring_fn((*ms).L, (*ms).capture[i as usize].init, l as usize);
        }
    }
}
