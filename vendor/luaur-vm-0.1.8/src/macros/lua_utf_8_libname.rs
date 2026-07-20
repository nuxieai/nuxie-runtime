pub const LUA_UTF8LIBNAME: &str = "utf8";

extern "C" {
    pub fn luaopen_utf8(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
