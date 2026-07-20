use core::ffi::{c_char, c_int};

use crate::functions::index_2_addr::index2addr;
use crate::macros::getstr::getstr;
use crate::macros::lua_s_updateatom::luaS_updateatom;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_tolstringatom(
    L: *mut lua_State,
    idx: c_int,
    len: *mut usize,
    atom: *mut c_int,
) -> *const c_char {
    let o: StkId = index2addr(L, idx);

    if !ttisstring!(o) {
        if !len.is_null() {
            *len = 0;
        }
        return core::ptr::null();
    }

    let s = tsvalue!(o);
    if !len.is_null() {
        *len = (*s).len as usize;
    }

    if !atom.is_null() {
        luaS_updateatom!(L, s as *mut crate::records::t_string::TString);
        *atom = (*s).atom as c_int;
    }

    getstr(s as *const crate::records::t_string::TString)
}
