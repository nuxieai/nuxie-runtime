use crate::functions::lua_l_buffinitsize::lua_l_buffinitsize;
use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_pushresultsize::lua_l_pushresultsize;
use crate::macros::uchar::uchar;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;
use core::ffi::c_int;

pub fn str_lower(L: *mut lua_State) -> c_int {
    unsafe {
        let mut len: usize = 0;
        let s = lua_l_checklstring(L, 1, &mut len);

        let mut b: LuaLStrbuf = LuaLStrbuf {
            p: core::ptr::null_mut(),
            end: core::ptr::null_mut(),
            L: core::ptr::null_mut(),
            storage: core::ptr::null_mut(),
            buffer: [0; 512],
        };
        let ptr = lua_l_buffinitsize(L, &mut b, len as usize);

        for i in 0..len as usize {
            unsafe {
                *ptr.add(i) = (uchar(*s.add(i) as c_int) as u8).to_ascii_lowercase() as c_char;
            }
        }

        lua_l_pushresultsize(&mut b, len as usize);
        1
    }
}
