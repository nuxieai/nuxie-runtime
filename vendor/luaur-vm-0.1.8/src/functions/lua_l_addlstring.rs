use crate::functions::extendstrbuf::extendstrbuf;
use crate::type_aliases::lua_l_strbuf::LuaLStrbuf;
use core::ffi::c_char;
use core::ptr::copy_nonoverlapping;

pub fn lua_l_addlstring(B: *mut LuaLStrbuf, s: *const c_char, len: usize) {
    unsafe {
        let current_buffer_size = (*B).end.offset_from((*B).p) as usize;
        if current_buffer_size < len as usize {
            extendstrbuf(B, len - current_buffer_size, -1);
        }

        copy_nonoverlapping(s as *const u8, (*B).p as *mut u8, len);
        (*B).p = (*B).p.offset(len as isize);
    }
}
