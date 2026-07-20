use crate::functions::digit::digit;
use crate::macros::lua_l_error::luaL_error;
use crate::records::header::Header;
use core::ffi::{c_char, c_int};

#[allow(non_snake_case)]
pub fn getnum(h: *mut Header, fmt: *mut *const c_char, df: i32) -> i32 {
    unsafe {
        if digit(**fmt as c_int) == 0 {
            // no number?
            df // return default value
        } else {
            let mut a: i32 = 0;
            let mut fmt_ptr = *fmt;
            loop {
                let digit_val = (*fmt_ptr as c_int) - b'0' as c_int;
                a = a * 10 + digit_val;
                fmt_ptr = fmt_ptr.add(1);

                if digit(*fmt_ptr as c_int) == 0 || a > (i32::MAX - 9) / 10 {
                    break;
                }
            }
            *fmt = fmt_ptr;

            if a > 1073741824 || digit(*fmt_ptr as c_int) != 0 {
                luaL_error!((*h).L, "size specifier is too large");
            }
            a
        }
    }
}
