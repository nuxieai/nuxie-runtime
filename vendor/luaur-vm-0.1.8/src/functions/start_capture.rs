use crate::functions::r#match::match_item;
use crate::macros::lua_l_error::luaL_error;
use crate::macros::lua_maxcaptures::LUA_MAXCAPTURES;
use crate::records::match_state::MatchState;
use core::ffi::c_char;

pub(crate) unsafe fn start_capture(
    ms: *mut MatchState,
    s: *const c_char,
    p: *const c_char,
    what: core::ffi::c_int,
) -> *const c_char {
    let level = (*ms).level;
    if level >= LUA_MAXCAPTURES {
        luaL_error!((*ms).L, "too many captures");
    }
    (*ms).capture[level as usize].init = s;
    (*ms).capture[level as usize].len = what as isize;
    (*ms).level = level + 1;
    let res = match_item(ms, s, p);
    if res.is_null() {
        (*ms).level -= 1;
    }
    res
}
