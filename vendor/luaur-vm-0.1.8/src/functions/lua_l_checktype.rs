use crate::functions::lua_type::lua_type;
use crate::functions::tag_error::tag_error;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_l_checktype(L: *mut lua_State, narg: core::ffi::c_int, t: core::ffi::c_int) {
    unsafe {
        if lua_type(L, narg) != t {
            tag_error(L, narg, t);
        }
    }
}
