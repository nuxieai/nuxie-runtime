use crate::macros::curr_func::curr_func;
use crate::macros::getstr::getstr;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State as lua_State_alias;

pub unsafe fn currfuncname(L: *mut lua_State_alias) -> *const core::ffi::c_char {
    let cl = if (*L).ci > (*L).base_ci {
        curr_func!(L)
    } else {
        core::ptr::null_mut()
    };

    let debugname = if !cl.is_null() && (*cl).isC != 0 {
        (&(*cl).inner.c).debugname
    } else {
        core::ptr::null()
    };

    if !debugname.is_null() && core::ffi::CStr::from_ptr(debugname).to_bytes() == b"__namecall" {
        if !(*L).namecall.is_null() {
            getstr((*L).namecall as *const TString)
        } else {
            core::ptr::null()
        }
    } else {
        debugname
    }
}
