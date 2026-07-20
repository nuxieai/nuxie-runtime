use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::ttisuserdata::ttisuserdata;
use crate::macros::uvalue::uvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_userdatatag(L: *mut lua_State, idx: c_int) -> c_int {
    let o: StkId = index2addr(L, idx);

    if ttisuserdata!(o) {
        // uvalue(o) returns a reference to the Udata struct.
        // The tag field is a uint8_t.
        (*uvalue!(o)).tag as c_int
    } else {
        -1
    }
}
