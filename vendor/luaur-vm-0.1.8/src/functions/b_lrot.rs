use crate::functions::b_rot::b_rot;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::type_aliases::lua_state::lua_State;

pub fn b_lrot(l: *mut lua_State) -> core::ffi::c_int {
    b_rot(l, lua_l_checkinteger(l, 2))
}
