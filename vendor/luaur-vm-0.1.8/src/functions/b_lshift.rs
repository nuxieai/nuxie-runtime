use crate::functions::b_shift::b_shift;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::type_aliases::lua_state::lua_State;

pub fn b_lshift(l: *mut lua_State) -> core::ffi::c_int {
    b_shift(l, lua_l_checkunsigned(l, 1), lua_l_checkinteger(l, 2))
}
