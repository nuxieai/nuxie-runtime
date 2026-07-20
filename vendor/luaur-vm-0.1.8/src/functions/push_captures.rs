use crate::functions::lua_l_checkstack::lua_l_checkstack;
use crate::functions::push_onecapture::push_onecapture;
use crate::records::match_state::MatchState;
use core::ffi::{c_char, c_int};

pub(crate) unsafe fn push_captures(
    ms: *mut MatchState,
    s: *const c_char,
    e: *const c_char,
) -> c_int {
    let nlevels = if (*ms).level == 0 && !s.is_null() {
        1
    } else {
        (*ms).level
    };

    lua_l_checkstack((*ms).L, nlevels, "too many captures");

    for i in 0..nlevels {
        push_onecapture(ms, i, s, e);
    }

    nlevels
}
