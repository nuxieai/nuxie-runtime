use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::functions::lua_pushunsigned::lua_pushunsigned;
use crate::type_aliases::b_uint::b_uint;
use crate::type_aliases::lua_state::lua_State;

pub fn b_swap(l: *mut lua_State) -> core::ffi::c_int {
    let n: b_uint = unsafe { lua_l_checkunsigned(l, 1) };
    let n = (n << 24) | ((n << 8) & 0xff0000) | ((n >> 8) & 0xff00) | (n >> 24);
    lua_pushunsigned(l, n);
    1
}
