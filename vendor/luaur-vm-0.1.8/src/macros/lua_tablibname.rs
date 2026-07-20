pub const LUA_TABLIBNAME: &str = "table";

extern "C" {
    pub fn luaopen_table(L: *mut crate::records::lua_state::LuaState) -> core::ffi::c_int;
}
