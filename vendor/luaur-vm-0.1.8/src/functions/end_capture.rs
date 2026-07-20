use crate::functions::capture_to_close::capture_to_close;
use crate::macros::cap_unfinished::CAP_UNFINISHED;
use crate::records::match_state::MatchState;
use core::ffi::c_char;

pub(crate) unsafe fn end_capture(
    ms: *mut MatchState,
    s: *const c_char,
    p: *const c_char,
) -> *const c_char {
    let l = capture_to_close(ms);

    // ms->capture[l].len = s - ms->capture[l].init; // close capture
    let init_ptr = (*ms).capture[l as usize].init;
    (*ms).capture[l as usize].len = (s as isize).wrapping_sub(init_ptr as isize);

    let res = crate::functions::r#match::match_item(ms, s, p);

    if res.is_null() {
        // ms->capture[l].len = CAP_UNFINISHED; // undo capture
        (*ms).capture[l as usize].len = CAP_UNFINISHED as isize;
    }

    res
}
