use crate::macros::luai_maxccalls::LUAI_MAXCCALLS;
use crate::records::match_state::MatchState;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) fn reprepstate(ms: *mut MatchState) {
    unsafe {
        (*ms).level = 0;
        LUAU_ASSERT!((*ms).matchdepth == LUAI_MAXCCALLS as i32);
    }
}
