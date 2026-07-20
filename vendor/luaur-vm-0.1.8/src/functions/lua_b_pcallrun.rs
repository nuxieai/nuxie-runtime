use crate::functions::lua_d_callint::lua_d_callint;
use crate::functions::lua_isyieldable::lua_isyieldable;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_b_pcallrun(L: *mut lua_State, ud: *mut core::ffi::c_void) {
    let func: StkId = ud as StkId;
    let preparereentry = lua_isyieldable(L) != 0;
    lua_d_callint(L, func, LUA_MULTRET, preparereentry);
}
