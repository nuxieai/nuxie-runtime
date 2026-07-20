use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::type_aliases::lua_state::lua_State;

pub fn buffer_len(L: *mut lua_State) -> core::ffi::c_int {
    let mut len: usize = 0;
    unsafe {
        lua_l_checkbuffer(L, 1, &mut len);
        lua_pushnumber(L, len as f64);
    }
    1
}
