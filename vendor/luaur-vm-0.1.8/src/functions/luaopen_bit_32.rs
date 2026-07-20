use crate::functions::lua_l_register::lua_l_register;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

use crate::functions::b_and::b_and;
use crate::functions::b_arshift::b_arshift;
use crate::functions::b_countlz::b_countlz;
use crate::functions::b_countrz::b_countrz;
use crate::functions::b_extract::b_extract;
use crate::functions::b_lrot::b_lrot;
use crate::functions::b_lshift::b_lshift;
use crate::functions::b_not::b_not;
use crate::functions::b_or::b_or;
use crate::functions::b_replace::b_replace;
use crate::functions::b_rrot::b_rrot;
use crate::functions::b_rshift::b_rshift;
use crate::functions::b_swap::b_swap;
use crate::functions::b_test::b_test;
use crate::functions::b_xor::b_xor;

pub unsafe fn luaopen_bit32(L: *mut lua_State) -> core::ffi::c_int {
    // Faithful port of bitlib[] in lbitlib.cpp (Lua name -> b_* function).
    let bitlib: [LuaLReg; 16] = [
        LuaLReg {
            name: c"arshift".as_ptr(),
            func: Some(b_arshift),
        },
        LuaLReg {
            name: c"band".as_ptr(),
            func: Some(b_and),
        },
        LuaLReg {
            name: c"bnot".as_ptr(),
            func: Some(b_not),
        },
        LuaLReg {
            name: c"bor".as_ptr(),
            func: Some(b_or),
        },
        LuaLReg {
            name: c"bxor".as_ptr(),
            func: Some(b_xor),
        },
        LuaLReg {
            name: c"btest".as_ptr(),
            func: Some(b_test),
        },
        LuaLReg {
            name: c"extract".as_ptr(),
            func: Some(b_extract),
        },
        LuaLReg {
            name: c"lrotate".as_ptr(),
            func: Some(b_lrot),
        },
        LuaLReg {
            name: c"lshift".as_ptr(),
            func: Some(b_lshift),
        },
        LuaLReg {
            name: c"replace".as_ptr(),
            func: Some(b_replace),
        },
        LuaLReg {
            name: c"rrotate".as_ptr(),
            func: Some(b_rrot),
        },
        LuaLReg {
            name: c"rshift".as_ptr(),
            func: Some(b_rshift),
        },
        LuaLReg {
            name: c"countlz".as_ptr(),
            func: Some(b_countlz),
        },
        LuaLReg {
            name: c"countrz".as_ptr(),
            func: Some(b_countrz),
        },
        LuaLReg {
            name: c"byteswap".as_ptr(),
            func: Some(b_swap),
        },
        LuaLReg {
            name: core::ptr::null(),
            func: None,
        },
    ];

    lua_l_register(L, c"bit32".as_ptr(), bitlib.as_ptr());

    1
}
