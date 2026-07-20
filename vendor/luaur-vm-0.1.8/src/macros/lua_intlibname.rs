pub const LUA_INTLIBNAME: &str = "integer";

extern "C" {
    pub fn luaopen_integer(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
