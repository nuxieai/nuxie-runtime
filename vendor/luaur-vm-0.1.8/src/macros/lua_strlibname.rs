pub const LUA_STRLIBNAME: &str = "string";

extern "C" {
    pub fn luaopen_string(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
