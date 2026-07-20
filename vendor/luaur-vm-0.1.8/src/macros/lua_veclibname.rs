pub const LUA_VECLIBNAME: &str = "vector";

extern "C" {
    pub fn luaopen_vector(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
