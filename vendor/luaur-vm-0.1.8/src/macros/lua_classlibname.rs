pub const LUA_CLASSLIBNAME: &str = "class";

extern "C" {
    pub fn luaopen_class(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
