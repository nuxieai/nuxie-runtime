use crate::functions::db_info::db_info;
use crate::functions::db_traceback::db_traceback;
use crate::functions::lua_l_register::lua_l_register;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

struct DblibWrapper([LuaLReg; 3]);
unsafe impl Sync for DblibWrapper {}

static DBLIB: DblibWrapper = DblibWrapper([
    LuaLReg {
        name: c"info".as_ptr(),
        func: Some(db_info),
    },
    LuaLReg {
        name: c"traceback".as_ptr(),
        func: Some(db_traceback),
    },
    LuaLReg {
        name: core::ptr::null(),
        func: None,
    },
]);

pub unsafe fn luaopen_debug(L: *mut lua_State) -> core::ffi::c_int {
    lua_l_register(L, c"debug".as_ptr(), DBLIB.0.as_ptr());
    1
}
