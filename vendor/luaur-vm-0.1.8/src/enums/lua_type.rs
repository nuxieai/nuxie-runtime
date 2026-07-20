#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum lua_Type {
    LUA_TNONE = -1,
    LUA_TNIL = 0,
    LUA_TBOOLEAN = 1,

    LUA_TLIGHTUSERDATA = 2,
    LUA_TNUMBER = 3,
    LUA_TINTEGER = 4,
    LUA_TVECTOR = 5,

    LUA_TSTRING = 6,

    LUA_TTABLE = 7,
    LUA_TFUNCTION = 8,
    LUA_TUSERDATA = 9,
    LUA_TTHREAD = 10,
    LUA_TBUFFER = 11,
    LUA_TCLASS = 12,
    LUA_TOBJECT = 13,

    LUA_TDEADKEY = 14,

    LUA_TPROTO = 15,
    LUA_TUPVAL = 16,
}

#[allow(non_upper_case_globals)]
pub const LUA_T_COUNT: lua_Type = lua_Type::LUA_TDEADKEY;

impl lua_Type {
    pub const LUA_TNIL: lua_Type = lua_Type::LUA_TNIL;
    pub const LUA_TNONE: lua_Type = lua_Type::LUA_TNONE;
    pub const LUA_TBOOLEAN: lua_Type = lua_Type::LUA_TBOOLEAN;
    pub const LUA_TLIGHTUSERDATA: lua_Type = lua_Type::LUA_TLIGHTUSERDATA;
    pub const LUA_TNUMBER: lua_Type = lua_Type::LUA_TNUMBER;
    pub const LUA_TINTEGER: lua_Type = lua_Type::LUA_TINTEGER;
    pub const LUA_TVECTOR: lua_Type = lua_Type::LUA_TVECTOR;
    pub const LUA_TSTRING: lua_Type = lua_Type::LUA_TSTRING;
    pub const LUA_TTABLE: lua_Type = lua_Type::LUA_TTABLE;
    pub const LUA_TFUNCTION: lua_Type = lua_Type::LUA_TFUNCTION;
    pub const LUA_TUSERDATA: lua_Type = lua_Type::LUA_TUSERDATA;
    pub const LUA_TTHREAD: lua_Type = lua_Type::LUA_TTHREAD;
    pub const LUA_TBUFFER: lua_Type = lua_Type::LUA_TBUFFER;
    pub const LUA_TCLASS: lua_Type = lua_Type::LUA_TCLASS;
    pub const LUA_TOBJECT: lua_Type = lua_Type::LUA_TOBJECT;
    pub const LUA_TDEADKEY: lua_Type = lua_Type::LUA_TDEADKEY;
    pub const LUA_TPROTO: lua_Type = lua_Type::LUA_TPROTO;
    pub const LUA_TUPVAL: lua_Type = lua_Type::LUA_TUPVAL;
}
