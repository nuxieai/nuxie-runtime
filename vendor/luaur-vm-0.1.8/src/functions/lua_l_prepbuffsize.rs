use crate::functions::extendstrbuf::extendstrbuf;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub fn lua_l_prepbuffsize(B: *mut LuaLStrbuf, size: usize) -> *mut c_char {
    unsafe {
        let current_p = (*B).p;
        let current_end = (*B).end;
        if (current_end as usize).wrapping_sub(current_p as usize) < size {
            extendstrbuf(
                B,
                size.wrapping_sub((current_end as usize).wrapping_sub(current_p as usize)),
                -1,
            )
        } else {
            current_p
        }
    }
}
