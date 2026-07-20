use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::lua_lutag_limit::LUA_LUTAG_LIMIT;
use crate::macros::setpvalue::setpvalue;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_pushlightuserdatatagged(
    L: *mut lua_State,
    p: *mut core::ffi::c_void,
    tag: core::ffi::c_int,
) {
    api_check!(L, (tag as u32) < LUA_LUTAG_LIMIT as u32);
    unsafe {
        setpvalue!((*L).top, p, tag);
    }
    api_incr_top!(L);
}
