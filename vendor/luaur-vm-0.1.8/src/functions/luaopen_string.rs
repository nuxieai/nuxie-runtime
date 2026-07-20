use crate::functions::createmetatable_lstrlib::createmetatable_mut;
use crate::functions::lua_l_register::lua_l_register;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

use crate::functions::gmatch::gmatch;
use crate::functions::str_byte::str_byte;
use crate::functions::str_char::str_char;
use crate::functions::str_find::str_find;
use crate::functions::str_format::str_format;
use crate::functions::str_gsub::str_gsub;
use crate::functions::str_len::str_len;
use crate::functions::str_lower::str_lower;
use crate::functions::str_match::str_match;
use crate::functions::str_pack::str_pack;
use crate::functions::str_packsize::str_packsize;
use crate::functions::str_rep::str_rep;
use crate::functions::str_reverse::str_reverse;
use crate::functions::str_split::str_split;
use crate::functions::str_sub::str_sub;
use crate::functions::str_unpack::str_unpack;
use crate::functions::str_upper::str_upper;

pub unsafe fn luaopen_string(l: *mut lua_State) -> core::ffi::c_int {
    // Faithful port of the `strlib[]` registration array in lstrlib.cpp:
    // {name, func} pairs ending in a {NULL, NULL} sentinel; lua_l_register
    // copies each into the `string` table.
    let strlib: [LuaLReg; 18] = [
        LuaLReg {
            name: c"byte".as_ptr(),
            func: Some(str_byte),
        },
        LuaLReg {
            name: c"char".as_ptr(),
            func: Some(str_char),
        },
        LuaLReg {
            name: c"find".as_ptr(),
            func: Some(str_find),
        },
        LuaLReg {
            name: c"format".as_ptr(),
            func: Some(str_format),
        },
        LuaLReg {
            name: c"gmatch".as_ptr(),
            func: Some(gmatch),
        },
        LuaLReg {
            name: c"gsub".as_ptr(),
            func: Some(str_gsub),
        },
        LuaLReg {
            name: c"len".as_ptr(),
            func: Some(str_len),
        },
        LuaLReg {
            name: c"lower".as_ptr(),
            func: Some(str_lower),
        },
        LuaLReg {
            name: c"match".as_ptr(),
            func: Some(str_match),
        },
        LuaLReg {
            name: c"rep".as_ptr(),
            func: Some(str_rep),
        },
        LuaLReg {
            name: c"reverse".as_ptr(),
            func: Some(str_reverse),
        },
        LuaLReg {
            name: c"sub".as_ptr(),
            func: Some(str_sub),
        },
        LuaLReg {
            name: c"upper".as_ptr(),
            func: Some(str_upper),
        },
        LuaLReg {
            name: c"split".as_ptr(),
            func: Some(str_split),
        },
        LuaLReg {
            name: c"pack".as_ptr(),
            func: Some(str_pack),
        },
        LuaLReg {
            name: c"packsize".as_ptr(),
            func: Some(str_packsize),
        },
        LuaLReg {
            name: c"unpack".as_ptr(),
            func: Some(str_unpack),
        },
        LuaLReg {
            name: core::ptr::null(),
            func: None,
        },
    ];

    lua_l_register(l, c"string".as_ptr(), strlib.as_ptr());
    createmetatable_mut(l);

    1
}
