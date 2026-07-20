//! Node: `cxx:Function:Luau.VM:VM/src/lstrlib.cpp:884:addquoted`
//!
//! Helper for `string.format("%q", s)` — append `s` to the buffer as a quoted,
//! escapable string literal: wrap in `"`, backslash-escape `"`/`\`/newline,
//! emit `\r` and `\000` for CR and NUL, pass everything else through.

use crate::functions::lua_l_addchar::lua_l_addchar;
use crate::functions::lua_l_addlstring::lua_l_addlstring;
use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_prepbuffsize::lua_l_prepbuffsize;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

pub fn addquoted(L: *mut lua_State, b: *mut LuaLStrbuf, arg: i32) {
    unsafe {
        let mut l: usize = 0;
        let mut s = lua_l_checklstring(L, arg, &mut l);

        lua_l_prepbuffsize(b, l + 2);

        lua_l_addchar(b, b'"' as c_char);
        while l != 0 {
            l -= 1;
            match *s as u8 {
                b'"' | b'\\' | b'\n' => {
                    lua_l_addchar(b, b'\\' as c_char);
                    lua_l_addchar(b, *s);
                }
                b'\r' => {
                    lua_l_addlstring(b, c"\\r".as_ptr(), 2);
                }
                b'\0' => {
                    lua_l_addlstring(b, c"\\000".as_ptr(), 4);
                }
                _ => {
                    lua_l_addchar(b, *s);
                }
            }
            s = s.add(1);
        }
        lua_l_addchar(b, b'"' as c_char);
    }
}
