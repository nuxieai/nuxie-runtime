#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum lua_Status {
    LUA_OK = 0,
    LUA_YIELD = 1,
    LUA_ERRRUN = 2,
    LUA_ERRSYNTAX = 3,
    LUA_ERRMEM = 4,
    LUA_ERRERR = 5,
    LUA_BREAK = 6,
}

pub use self::lua_Status as LuaStatus;

impl lua_Status {
    pub const LUA_OK: lua_Status = lua_Status::LUA_OK;
    pub const LUA_YIELD: lua_Status = lua_Status::LUA_YIELD;
    pub const LUA_ERRRUN: lua_Status = lua_Status::LUA_ERRRUN;
    pub const LUA_ERRSYNTAX: lua_Status = lua_Status::LUA_ERRSYNTAX;
    pub const LUA_ERRMEM: lua_Status = lua_Status::LUA_ERRMEM;
    pub const LUA_ERRERR: lua_Status = lua_Status::LUA_ERRERR;
    pub const LUA_BREAK: lua_Status = lua_Status::LUA_BREAK;
}

impl Default for lua_Status {
    fn default() -> Self {
        lua_Status::LUA_OK
    }
}
