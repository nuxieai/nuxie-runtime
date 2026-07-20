//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:529:luaL_addvalueany`
//! Source: `VM/src/laux.cpp:529-582` (hand-ported)

use crate::enums::lua_type::lua_Type;
use crate::functions::lua_l_addlstring::lua_l_addlstring;
use crate::functions::lua_l_addvalue::lua_l_addvalue;
use crate::functions::lua_l_tolstring::lua_l_tolstring;
use crate::functions::lua_toboolean::lua_toboolean;
use crate::functions::lua_tointeger_64::lua_tointeger_64;
use crate::functions::lua_tolstring::lua_tolstring;
use crate::functions::lua_tonumberx::lua_tonumberx;
use crate::functions::lua_type::lua_type;
use crate::functions::luai_int_2_str::luai_int2str;
use crate::functions::luai_num_2_str::luai_num2str;
use crate::macros::luai_maxint_2_str::LUAI_MAXINT2STR;
use crate::macros::luai_maxnum_2_str::LUAI_MAXNUM2STR;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use core::ffi::{c_char, c_int};

/// C++ `void luaL_addvalueany(luaL_Strbuf *B, int idx)` —
/// converts the value at stack index `idx` to its string representation
/// and appends it to buffer `B`.
pub fn lua_l_addvalueany(B: *mut LuaLStrbuf, idx: c_int) {
    unsafe {
        let L = (*B).L;

        match lua_type(L, idx) {
            x if x == lua_Type::LUA_TNONE as c_int => {
                panic!("expected value");
            }
            x if x == lua_Type::LUA_TNIL as c_int => {
                lua_l_addlstring(B, c"nil".as_ptr(), 3);
            }
            x if x == lua_Type::LUA_TBOOLEAN as c_int => {
                if lua_toboolean(L, idx) != 0 {
                    lua_l_addlstring(B, c"true".as_ptr(), 4);
                } else {
                    lua_l_addlstring(B, c"false".as_ptr(), 5);
                }
            }
            x if x == lua_Type::LUA_TNUMBER as c_int => {
                let mut isnum: c_int = 0;
                let n = lua_tonumberx(L, idx, &mut isnum);
                let mut s = [0 as c_char; LUAI_MAXNUM2STR as usize];
                let e = luai_num2str(s.as_mut_ptr(), n);
                lua_l_addlstring(B, s.as_ptr(), e.offset_from(s.as_ptr()) as usize);
            }
            x if x == lua_Type::LUA_TSTRING as c_int => {
                let mut len: usize = 0;
                let s = lua_tolstring(L, idx, &mut len);
                lua_l_addlstring(B, s, len);
            }
            x if x == lua_Type::LUA_TINTEGER as c_int => {
                let n = lua_tointeger_64(L, idx, core::ptr::null_mut());
                let mut s = [0 as c_char; LUAI_MAXINT2STR as usize];
                let e = luai_int2str(s.as_mut_ptr(), n);
                lua_l_addlstring(B, s.as_ptr(), e.offset_from(s.as_ptr()) as usize);
            }
            _ => {
                // note: luaL_addlstring assumes box is stored at top of stack, so we can't call it here
                // instead we use luaL_addvalue which will take the string from the top of the stack and add that
                let mut len: usize = 0;
                lua_l_tolstring(L, idx, &mut len);
                lua_l_addvalue(B);
            }
        }
    }
}
