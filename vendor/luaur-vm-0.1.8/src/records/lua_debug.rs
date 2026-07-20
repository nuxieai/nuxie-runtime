//! Node: `cxx:Record:Luau.VM:VM/include/lua.h:488:lua_debug`
//! Source: `VM/include/lua.h:488-502` (hand-ported; was a `_private: ()`
//! placeholder — the lying-record class)

/// C++ `struct lua_Debug` — activation record. `LUA_IDSIZE` = 256
/// (luaconf.h:71).
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LuaDebug {
    pub name: *const core::ffi::c_char,      // (n)
    pub what: *const core::ffi::c_char,      // (s) `Lua', `C', `main', `tail'
    pub source: *const core::ffi::c_char,    // (s)
    pub short_src: *const core::ffi::c_char, // (s)
    pub linedefined: core::ffi::c_int,       // (s)
    pub currentline: core::ffi::c_int,       // (l)
    pub nupvals: u8,                         // (u) number of upvalues
    pub nparams: u8,                         // (a) number of parameters
    pub isvararg: core::ffi::c_char,         // (a)
    /// only valid in luau_callhook
    pub userdata: *mut core::ffi::c_void,

    pub ssbuf: [core::ffi::c_char; 256],
}

#[allow(non_camel_case_types)]
pub type lua_Debug = LuaDebug;
