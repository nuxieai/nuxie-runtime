use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::ttislightuserdata::ttislightuserdata;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_tolightuserdata(L: *mut lua_State, idx: c_int) -> *mut core::ffi::c_void {
    let o: StkId = index2addr(L, idx);

    if !ttislightuserdata!(o) {
        core::ptr::null_mut()
    } else {
        // Based on the provided context for pvalue(o) in lobject.h:
        // #define pvalue(o) check_exp(ttislightuserdata(o), (o)->value.p)
        // and the example lua_touserdata which accesses (*o).value.p directly.
        (*o).value.p
    }
}
