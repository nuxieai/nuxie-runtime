use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::functions::lua_setfield::lua_setfield;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

pub fn setboolfield(L: *mut lua_State, key: &str, value: i32) {
    if value < 0 {
        return;
    }

    unsafe {
        lua_pushboolean(L, value);

        let key_bytes = key.as_bytes();
        let mut buf = key_bytes.to_vec();
        buf.push(0);
        let key_c: *const c_char = buf.as_ptr() as *const c_char;

        lua_setfield(L, -2, key_c);
    }
}
