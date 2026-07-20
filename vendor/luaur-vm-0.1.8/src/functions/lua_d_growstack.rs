use crate::functions::lua_d_reallocstack::lua_d_reallocstack;
use crate::macros::getgrownstacksize::getgrownstacksize;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_d_growstack(l: *mut lua_State, n: core::ffi::c_int) {
    unsafe {
        lua_d_reallocstack(l, getgrownstacksize(l, n), 0);
    }
}
