pub const LUA_DBLIBNAME: &str = "debug";

extern "C" {
    pub fn luaopen_debug(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
