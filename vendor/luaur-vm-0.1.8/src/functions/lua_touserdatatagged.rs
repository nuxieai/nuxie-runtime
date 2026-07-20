use core::ffi::{c_int, c_void};

use crate::functions::index_2_addr::index2addr;
use crate::macros::ttisuserdata::ttisuserdata;
use crate::macros::uvalue::uvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_touserdatatagged(L: *mut lua_State, idx: c_int, tag: c_int) -> *mut c_void {
    let o: StkId = index2addr(L, idx);

    // uvalue(o)->tag is a u8 in the Udata struct, while tag is a c_int (i32).
    // We cast the struct field to i32 to perform the comparison.
    if ttisuserdata!(o) && (uvalue!(o).tag as i32) == tag {
        uvalue!(o).data.as_ptr() as *mut c_void
    } else {
        core::ptr::null_mut()
    }
}
