use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::lightuserdatatag::lightuserdatatag;
use crate::macros::ttislightuserdata::ttislightuserdata;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_lightuserdatatag(L: *mut lua_State, idx: c_int) -> c_int {
    let o: StkId = index2addr(L, idx);

    if ttislightuserdata!(o) {
        lightuserdatatag!(o)
    } else {
        -1
    }
}
