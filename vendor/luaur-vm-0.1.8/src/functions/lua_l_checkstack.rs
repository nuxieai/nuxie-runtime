use crate::functions::lua_checkstack::lua_checkstack;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub unsafe fn lua_l_checkstack(L: *mut lua_State, space: c_int, mes: &str) {
    if lua_checkstack(L, space) == 0 {
        lua_l_error_l(
            L,
            c"stack overflow (%s)".as_ptr(),
            format_args!("stack overflow ({})", mes),
        );
    }
}
