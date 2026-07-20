use core::ffi::{c_char, c_int};

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_v_tostring::lua_v_tostring;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::svalue::svalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisstring::ttisstring;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_tolstring(L: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char {
    let mut o: StkId = index2addr(L, idx);

    if !ttisstring!(o) {
        lua_c_threadbarrier_lapi(L);
        if lua_v_tostring(L, o) == 0 {
            if !len.is_null() {
                *len = 0;
            }
            return core::ptr::null();
        }
        luaC_checkGC!(L);
        o = index2addr(L, idx);
    }

    if !len.is_null() {
        *len = (*tsvalue!(o)).len as usize;
    }

    svalue!(o)
}
