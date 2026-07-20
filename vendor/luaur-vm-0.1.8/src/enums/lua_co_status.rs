#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum lua_CoStatus {
    LUA_CORUN = 0,
    LUA_COSUS = 1,
    LUA_CONOR = 2,
    LUA_COFIN = 3,
    LUA_COERR = 4,
}

#[allow(non_upper_case_globals)]
impl lua_CoStatus {
    pub const LUA_CORUN: Self = Self::LUA_CORUN;
    pub const LUA_COSUS: Self = Self::LUA_COSUS;
    pub const LUA_CONOR: Self = Self::LUA_CONOR;
    pub const LUA_COFIN: Self = Self::LUA_COFIN;
    pub const LUA_COERR: Self = Self::LUA_COERR;
}

#[allow(non_camel_case_types)]
pub type LuaCoStatus = lua_CoStatus;
