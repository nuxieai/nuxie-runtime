use crate::functions::check_capture::check_capture;
use crate::records::match_state::MatchState;
use core::ffi::{c_char, c_int};

pub(crate) unsafe fn match_capture(
    ms: *mut MatchState,
    s: *const c_char,
    l: c_int,
) -> *const c_char {
    let l = check_capture(ms, l);
    let len = (*ms).capture[l as usize].len as usize;

    if ((*ms).src_end as usize).wrapping_sub(s as usize) >= len
        && core::slice::from_raw_parts((*ms).capture[l as usize].init as *const u8, len)
            == core::slice::from_raw_parts(s as *const u8, len)
    {
        s.add(len)
    } else {
        core::ptr::null()
    }
}
