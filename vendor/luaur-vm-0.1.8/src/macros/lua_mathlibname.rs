pub const LUA_MATHLIBNAME: &str = "math";

extern "C" {
    pub fn luaopen_math(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
