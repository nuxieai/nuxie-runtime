//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:297:libsize`
//! Source: `VM/src/laux.cpp:297-302` (hand-ported)

use core::ffi::c_int;

use crate::records::lua_l_reg::LuaLReg;

pub unsafe fn libsize(mut l: *const LuaLReg) -> c_int {
    let mut size = 0;
    while !(*l).name.is_null() {
        size += 1;
        l = l.add(1);
    }
    size
}
