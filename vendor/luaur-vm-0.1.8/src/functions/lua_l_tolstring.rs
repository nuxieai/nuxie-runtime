//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:616:luaL_tolstring`
//! Source: `VM/src/laux.cpp:616-679` (hand-ported)

use core::ffi::{c_char, c_int};

use crate::enums::lua_type::lua_Type;
use crate::functions::lua_encodepointer::lua_encodepointer;
use crate::functions::lua_l_callmeta::lua_l_callmeta;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::functions::lua_l_typename::lua_l_typename;
use crate::functions::lua_pushfstring_l::lua_pushfstring_l;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_pushstring::lua_pushstring;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_toboolean::lua_toboolean;
use crate::functions::lua_tointeger_64::lua_tointeger_64;
use crate::functions::lua_tolstring::lua_tolstring;
use crate::functions::lua_tonumberx::lua_tonumberx;
use crate::functions::lua_topointer::lua_topointer;
use crate::functions::lua_tovector::lua_tovector;
use crate::functions::lua_type::lua_type;
use crate::functions::luai_int_2_str::luai_int2str;
use crate::functions::luai_num_2_str::luai_num2str;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::macros::luai_maxint_2_str::LUAI_MAXINT2STR;
use crate::macros::luai_maxnum_2_str::LUAI_MAXNUM2STR;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_tolstring(L: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char {
    if lua_l_callmeta(L, idx, c"__tostring".as_ptr()) != 0 {
        let s = lua_tolstring(L, -1, len);
        if s.is_null() {
            lua_l_error_l(
                L,
                c"'__tostring' must return a string".as_ptr(),
                format_args!("'__tostring' must return a string"),
            );
        }
        return s;
    }

    match lua_type(L, idx) {
        x if x == lua_Type::LUA_TNIL as c_int => {
            lua_pushlstring(L, c"nil".as_ptr(), 3);
        }
        x if x == lua_Type::LUA_TBOOLEAN as c_int => {
            lua_pushstring(
                L,
                if lua_toboolean(L, idx) != 0 {
                    c"true".as_ptr()
                } else {
                    c"false".as_ptr()
                },
            );
        }
        x if x == lua_Type::LUA_TNUMBER as c_int => {
            let mut isnum = 0;
            let n = lua_tonumberx(L, idx, &mut isnum);
            let mut s = [0 as c_char; LUAI_MAXNUM2STR as usize];
            let e = luai_num2str(s.as_mut_ptr(), n);
            lua_pushlstring(L, s.as_ptr(), e.offset_from(s.as_ptr()) as usize);
        }
        x if x == lua_Type::LUA_TVECTOR as c_int => {
            let v = lua_tovector(L, idx);
            let mut s = [0 as c_char; (LUAI_MAXNUM2STR as usize) * (LUA_VECTOR_SIZE as usize)];
            let mut e = s.as_mut_ptr();
            let mut i = 0;
            while i < LUA_VECTOR_SIZE {
                if i != 0 {
                    *e = b',' as c_char;
                    e = e.add(1);
                    *e = b' ' as c_char;
                    e = e.add(1);
                }
                e = luai_num2str(e, *v.add(i as usize) as f64);
                i += 1;
            }
            lua_pushlstring(L, s.as_ptr(), e.offset_from(s.as_ptr()) as usize);
        }
        x if x == lua_Type::LUA_TSTRING as c_int => {
            lua_pushvalue(L, idx);
        }
        x if x == lua_Type::LUA_TINTEGER as c_int => {
            let l = lua_tointeger_64(L, idx, core::ptr::null_mut());
            let mut s = [0 as c_char; LUAI_MAXINT2STR as usize];
            let e = luai_int2str(s.as_mut_ptr(), l);
            lua_pushlstring(L, s.as_ptr(), e.offset_from(s.as_ptr()) as usize);
        }
        _ => {
            let ptr = lua_topointer(L, idx);
            let enc = lua_encodepointer(L, ptr as usize);
            let name = core::ffi::CStr::from_ptr(lua_l_typename(L, idx)).to_string_lossy();
            lua_pushfstring_l(
                L,
                c"%s: 0x%016llx".as_ptr(),
                format_args!("{}: 0x{:016x}", name, enc),
            );
        }
    }

    lua_tolstring(L, -1, len)
}
