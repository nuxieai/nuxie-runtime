//! Node: `cxx:Function:Luau.VM:VM/src/lstrlib.cpp:47:str_reverse`
//!
//! `string.reverse` — copy the argument bytes into a fresh buffer back-to-front.

use crate::functions::lua_l_buffinitsize::lua_l_buffinitsize;
use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_pushresultsize::lua_l_pushresultsize;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub fn str_reverse(L: *mut lua_State) -> c_int {
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
        let ptr = lua_l_buffinitsize(L, &mut b, len);

        for i in 0..len {
            *ptr.add(i) = *s.add(len - 1 - i);
        }

        lua_l_pushresultsize(&mut b, len);
        1
    }
}
