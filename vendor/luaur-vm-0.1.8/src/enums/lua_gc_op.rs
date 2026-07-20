#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum lua_GCOp {
    LUA_GCSTOP = 0,
    LUA_GCRESTART = 1,
    LUA_GCCOLLECT = 2,
    LUA_GCCOUNT = 3,
    LUA_GCCOUNTB = 4,
    LUA_GCISRUNNING = 5,
    LUA_GCSTEP = 6,
    LUA_GCSETGOAL = 7,
    LUA_GCSETSTEPMUL = 8,
    LUA_GCSETSTEPSIZE = 9,
}

#[allow(non_upper_case_globals)]
impl lua_GCOp {
    pub const LUA_GCSTOP: Self = Self::LUA_GCSTOP;
    pub const LUA_GCRESTART: Self = Self::LUA_GCRESTART;
    pub const LUA_GCCOLLECT: Self = Self::LUA_GCCOLLECT;
    pub const LUA_GCCOUNT: Self = Self::LUA_GCCOUNT;
    pub const LUA_GCCOUNTB: Self = Self::LUA_GCCOUNTB;
    pub const LUA_GCISRUNNING: Self = Self::LUA_GCISRUNNING;
    pub const LUA_GCSTEP: Self = Self::LUA_GCSTEP;
    pub const LUA_GCSETGOAL: Self = Self::LUA_GCSETGOAL;
    pub const LUA_GCSETSTEPMUL: Self = Self::LUA_GCSETSTEPMUL;
    pub const LUA_GCSETSTEPSIZE: Self = Self::LUA_GCSETSTEPSIZE;
}
