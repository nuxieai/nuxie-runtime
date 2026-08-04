use crate::functions::cstring_builtin_folding_alt_b::cstring_c_char_usize;
use crate::records::constant::Constant;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub fn cstring_c_char(v: *const c_char) -> Constant {
    let mut len = 0;
    unsafe {
        while *v.add(len) != 0 {
            len += 1;
        }
    }
    cstring_c_char_usize(v, len)
}
