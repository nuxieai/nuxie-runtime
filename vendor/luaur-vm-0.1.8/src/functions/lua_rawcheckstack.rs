use crate::macros::api_check::api_check;
use crate::macros::expandstacklimit::expandstacklimit;
use crate::macros::lua_d_checkstack::luaD_checkstack;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_rawcheckstack(L: *mut lua_State, size: core::ffi::c_int) {
    api_check!(L, size >= 0);

    unsafe {
        luaD_checkstack!(L, size);
        expandstacklimit!(L, (*L).top.wrapping_add(size as usize));
    }
}
