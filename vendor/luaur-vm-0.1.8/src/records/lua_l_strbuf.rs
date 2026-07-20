//! Node: lua_l_strbuf
//! Source: `VM/include/lualib.h` (lualib.h:86-98, hand-ported)

use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

// luaconf.h:96
pub const LUA_BUFFERSIZE: usize = 512;

#[repr(C)]
#[derive(Debug)]
pub struct LuaLStrbuf {
    pub p: *mut c_char,   // current position in buffer
    pub end: *mut c_char, // end of the current buffer
    pub L: *mut lua_State,
    pub storage: *mut TString,
    pub buffer: [c_char; LUA_BUFFERSIZE],
}

#[allow(non_camel_case_types)]
pub type luaL_Strbuf = LuaLStrbuf;
// compatibility typedef: called luaL_Buffer in Lua headers
#[allow(non_camel_case_types)]
pub type luaL_Buffer = LuaLStrbuf;
