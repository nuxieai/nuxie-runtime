use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::records::lua_state::lua_State;
use crate::records::t_string::TString;
use core::ffi::{c_char, CStr};

#[allow(non_snake_case)]
pub fn luaS_new(l: *mut lua_State, s: *const c_char) -> *mut TString {
    unsafe {
        let len = CStr::from_ptr(s).to_bytes().len();
        luaS_newlstr(l, s, len)
    }
}

#[allow(non_snake_case)]
pub fn luaS_newliteral(l: *mut lua_State, s: *const c_char) -> *mut TString {
    unsafe {
        let len = CStr::from_ptr(s).to_bytes().len();
        luaS_newlstr(l, s, len)
    }
}
