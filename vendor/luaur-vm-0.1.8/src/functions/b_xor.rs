use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::functions::lua_pushunsigned::lua_pushunsigned;
use crate::macros::trim::trim;
use crate::type_aliases::b_uint::b_uint;
use crate::type_aliases::lua_state::lua_State;

pub fn b_xor(l: *mut lua_State) -> core::ffi::c_int {
    let n = unsafe { lua_gettop(l) };
    let mut r: b_uint = 0;

    for i in 1..=n {
        r ^= lua_l_checkunsigned(l, i);
    }

    lua_pushunsigned(l, trim(r));
    1
}
