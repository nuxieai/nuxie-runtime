use crate::macros::cap_unfinished::CAP_UNFINISHED;
use crate::macros::lua_l_error::luaL_error;
use crate::records::match_state::MatchState;
use core::ffi::c_int;

pub(crate) unsafe fn check_capture(ms: *mut MatchState, mut l: c_int) -> c_int {
    l -= '1' as c_int;
    if l < 0 || l >= (*ms).level || (*ms).capture[l as usize].len == CAP_UNFINISHED as isize {
        let fmt = "invalid capture index %d";
        let fmt_ptr = fmt.as_ptr() as *const core::ffi::c_char;
        crate::functions::lua_l_error_l::lua_l_error_l(
            (*ms).L,
            fmt_ptr,
            core::format_args!("{}", l + 1),
        );
    }
    l
}
