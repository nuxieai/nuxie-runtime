use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_tovector(L: *mut lua_State, idx: c_int) -> *const f32 {
    let o: StkId = index2addr(L, idx);
    if !ttisvector!(o) {
        core::ptr::null()
    } else {
        vvalue!(o).as_ptr()
    }
}
