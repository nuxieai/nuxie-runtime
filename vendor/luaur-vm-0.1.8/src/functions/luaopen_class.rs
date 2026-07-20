use crate::functions::class_classof::class_classof;
use crate::functions::class_isinstance::class_isinstance;
use crate::functions::lua_l_register::lua_l_register;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn luaopen_class(L: *mut lua_State) -> core::ffi::c_int {
    let class_lib: [LuaLReg; 3] = [
        LuaLReg {
            name: c"isinstance".as_ptr(),
            func: Some(class_isinstance),
        },
        LuaLReg {
            name: c"classof".as_ptr(),
            func: Some(class_classof),
        },
        LuaLReg {
            name: core::ptr::null(),
            func: None,
        },
    ];

    lua_l_register(L, c"class".as_ptr(), class_lib.as_ptr());
    1
}
