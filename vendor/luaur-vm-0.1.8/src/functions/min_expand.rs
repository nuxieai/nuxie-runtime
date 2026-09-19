use crate::functions::r#match::match_item as match_fn;
use crate::functions::singlematch::singlematch;
use crate::records::match_state::MatchState;
use core::ffi::c_char;

pub(crate) unsafe fn min_expand(
    ms: *mut MatchState,
    mut s: *const c_char,
    p: *const c_char,
    ep: *const c_char,
) -> crate::records::lua_exception::LuaResult<*const c_char> {
    loop {
        let res = match_fn(ms, s, ep.add(1))?;

        if !res.is_null() {
            return Ok(res);
        } else if singlematch(ms, s, p, ep) != 0 {
            // try with one more repetition
            s = s.add(1);
        } else {
            return Ok(core::ptr::null());
        }
    }
}
