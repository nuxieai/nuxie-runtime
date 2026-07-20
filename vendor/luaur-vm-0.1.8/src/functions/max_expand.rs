use crate::functions::r#match::match_item;
use crate::functions::singlematch::singlematch;
use crate::records::match_state::MatchState;
use core::ffi::c_char;

pub(crate) unsafe fn max_expand(
    ms: *mut MatchState,
    s: *const c_char,
    p: *const c_char,
    ep: *const c_char,
) -> *const c_char {
    let mut i: isize = 0; // counts maximum expand for item

    // while (singlematch(ms, s + i, p, ep))
    //     i++;
    while singlematch(ms, s.offset(i), p, ep) != 0 {
        i += 1;
    }

    // keeps trying to match with the maximum repetitions
    while i >= 0 {
        let res = match_item(ms, s.offset(i), ep.offset(1));

        if !res.is_null() {
            return res;
        }

        i -= 1; // else didn't match; reduce 1 repetition to try again
    }

    core::ptr::null()
}
