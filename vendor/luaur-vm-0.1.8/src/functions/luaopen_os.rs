use crate::functions::lua_l_register::lua_l_register;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

use crate::functions::os_clock::os_clock;
use crate::functions::os_date::os_date;
use crate::functions::os_difftime::os_difftime;
use crate::functions::os_time::os_time;

pub unsafe fn luaopen_os(L: *mut lua_State) -> core::ffi::c_int {
    // Faithful port of syslib[] in loslib.cpp.
    let syslib: [LuaLReg; 5] = [
        LuaLReg {
            name: c"clock".as_ptr(),
            func: Some(os_clock),
        },
        LuaLReg {
            name: c"date".as_ptr(),
            func: Some(os_date),
        },
        LuaLReg {
            name: c"difftime".as_ptr(),
            func: Some(os_difftime),
        },
        LuaLReg {
            name: c"time".as_ptr(),
            func: Some(os_time),
        },
        LuaLReg {
            name: core::ptr::null(),
            func: None,
        },
    ];

    lua_l_register(L, c"os".as_ptr(), syslib.as_ptr());
    1
}
