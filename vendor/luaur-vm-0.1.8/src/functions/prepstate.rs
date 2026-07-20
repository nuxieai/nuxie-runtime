use crate::macros::luai_maxccalls::LUAI_MAXCCALLS;
use crate::records::match_state::MatchState;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_void};

#[allow(non_snake_case)]
pub(crate) unsafe fn prepstate(
    ms: *mut MatchState,
    l: *mut lua_State,
    s: *const c_char,
    ls: usize,
    p: *const c_char,
    lp: usize,
) {
    (*ms).L = l;
    (*ms).matchdepth = LUAI_MAXCCALLS;
    (*ms).src_init = s;
    (*ms).src_end = s.add(ls);
    (*ms).p_end = p.add(lp);
}
