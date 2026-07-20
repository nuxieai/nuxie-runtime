use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::l_isfalse::l_isfalse;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_toboolean(L: *mut lua_State, idx: c_int) -> c_int {
    let o: *const crate::type_aliases::t_value::TValue = index2addr(L, idx);
    (!l_isfalse!(o)) as c_int
}
