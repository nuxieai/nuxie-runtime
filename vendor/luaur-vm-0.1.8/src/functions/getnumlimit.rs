use crate::functions::getnum::getnum;
use crate::macros::lua_l_error::luaL_error;
use crate::records::header::Header;
use core::ffi::{c_char, c_int};

pub fn getnumlimit(h: *mut Header, fmt: *mut *const c_char, df: c_int) -> c_int {
    let sz = getnum(h, fmt, df);
    if sz > 16 || sz <= 0 {
        unsafe { luaL_error!((*h).L, "integral size ({}) out of limits [1,{}]", sz, 16,) };
    }
    sz
}
