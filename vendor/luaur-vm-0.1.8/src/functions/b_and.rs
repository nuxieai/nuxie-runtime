use crate::functions::andaux::andaux;
use crate::functions::lua_pushunsigned::lua_pushunsigned;
use crate::type_aliases::lua_state::lua_State;

pub fn b_and(l: *mut lua_State) -> core::ffi::c_int {
    let r = andaux(l);
    lua_pushunsigned(l, r);
    1
}
