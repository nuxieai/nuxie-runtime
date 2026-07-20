use core::ffi::{c_char, c_int};

use crate::macros::api_check::api_check;
use crate::macros::lua_tnone::LUA_TNONE;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_typename(L: *mut lua_State, t: c_int) -> *const c_char {
    api_check!(
        L,
        t >= LUA_TNONE && t < crate::enums::lua_type::LUA_T_COUNT as c_int
    );

    match t {
        LUA_TNONE => c"no value".as_ptr(),
        0 => c"nil".as_ptr(),
        1 => c"boolean".as_ptr(),
        2 => c"userdata".as_ptr(),
        3 => c"number".as_ptr(),
        4 => c"integer".as_ptr(),
        5 => c"vector".as_ptr(),
        6 => c"string".as_ptr(),
        7 => c"table".as_ptr(),
        8 => c"function".as_ptr(),
        9 => c"userdata".as_ptr(),
        10 => c"thread".as_ptr(),
        11 => c"buffer".as_ptr(),
        12 => c"class".as_ptr(),
        13 => c"object".as_ptr(),
        _ => core::ptr::null(),
    }
}
