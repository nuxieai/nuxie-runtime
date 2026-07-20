use crate::functions::lua_l_register::lua_l_register;
use crate::functions::lua_pushinteger_64::lua_pushinteger_64;
use crate::functions::lua_setfield::lua_setfield;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn luaopen_integer(l: *mut lua_State) -> core::ffi::c_int {
    // Register the integer library functions
    // Note: int64lib is defined in lintlib.cpp; in this translation context we use the local static.
    lua_l_register(l, c"int64".as_ptr(), INT64LIB.as_ptr());

    // Push LLONG_MAX and set it as "maxsigned"
    lua_pushinteger_64(l, i64::MAX);
    lua_setfield(l, -2, c"maxsigned".as_ptr());

    // Push LLONG_MIN and set it as "minsigned"
    lua_pushinteger_64(l, i64::MIN);
    lua_setfield(l, -2, c"minsigned".as_ptr());

    1
}

// Define the int64lib luaL_Reg table locally as in the C++ source.
// We use a Sync wrapper or just raw pointers in a way that satisfies the compiler for statics.
struct SyncLuaLReg([LuaLReg; 1]);
unsafe impl Sync for SyncLuaLReg {}

static INT64LIB: SyncLuaLReg = SyncLuaLReg([LuaLReg {
    name: core::ptr::null(),
    func: None,
}]);

impl core::ops::Deref for SyncLuaLReg {
    type Target = [LuaLReg; 1];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
