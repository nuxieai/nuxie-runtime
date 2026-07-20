use crate::functions::lua_tothread::lua_tothread;
use crate::macros::lua_isthread::lua_isthread;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub fn getthread(l: *mut lua_State, arg: *mut c_int) -> *mut lua_State {
    if unsafe { lua_isthread!(l, 1) } {
        unsafe {
            *arg = 1;
            lua_tothread(l, 1)
        }
    } else {
        unsafe {
            *arg = 0;
        }
        l
    }
}
