use core::ffi::c_int;

use crate::enums::lua_type::lua_Type;
use crate::functions::index_2_addr::index2addr;
use crate::macros::ttislightuserdata::ttislightuserdata;
use crate::macros::ttisuserdata::ttisuserdata;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_isuserdata(L: *mut lua_State, idx: c_int) -> c_int {
    let o: *const TValue = index2addr(L, idx);
    (ttisuserdata!(o) || ttislightuserdata!(o)) as c_int
}
