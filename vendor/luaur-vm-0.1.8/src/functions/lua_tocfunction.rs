use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::cast_to::cast_to;
use crate::macros::clvalue::clvalue;
use crate::macros::iscfunction::iscfunction;
use crate::type_aliases::lua_c_function::lua_CFunction;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_tocfunction(L: *mut lua_State, idx: c_int) -> lua_CFunction {
    let o: StkId = index2addr(L, idx);

    if !iscfunction!(o) {
        None
    } else {
        let cl = clvalue!(o);
        let c = core::ptr::addr_of!((*cl).inner.c).cast::<crate::records::closure::CClosure>();
        let f = (*c).f;
        cast_to!(lua_CFunction, f)
    }
}
