use crate::functions::lua_newstate::lua_newstate;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;

pub fn lua_l_newstate() -> *mut lua_State {
    unsafe {
        lua_newstate(
            Some(crate::functions::l_alloc::l_alloc),
            core::ptr::null_mut(),
        )
    }
}
