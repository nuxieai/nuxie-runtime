use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::iscfunction::iscfunction;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_iscfunction(L: *mut lua_State, idx: c_int) -> c_int {
    let o: StkId = index2addr(L, idx);
    if iscfunction!(o) {
        1
    } else {
        0
    }
}
