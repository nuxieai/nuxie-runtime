pub const LUA_OSLIBNAME: &str = "os";

extern "C" {
    pub fn luaopen_os(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
