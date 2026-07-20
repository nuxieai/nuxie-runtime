use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::functions::lua_pushunsigned::lua_pushunsigned;
use crate::macros::trim::trim;
use crate::type_aliases::b_uint::b_uint;
use crate::type_aliases::lua_state::lua_State;

pub fn b_not(l: *mut lua_State) -> core::ffi::c_int {
    let r = unsafe { !(lua_l_checkunsigned(l, 1)) };
    lua_pushunsigned(l, trim(r));
    1
}
