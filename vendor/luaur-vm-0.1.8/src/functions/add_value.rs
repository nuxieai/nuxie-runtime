//! Node: `cxx:Function:Luau.VM:VM/src/lstrlib.cpp:796:add_value`
//!
//! `string.gsub` replacement dispatch for one match: a function replacement is
//! called with the captures, a table replacement is indexed by the first
//! capture, and a string/number replacement goes through `add_s`. A falsy or
//! non-string result falls back to the original matched text.

use crate::enums::lua_type::lua_Type;
use crate::functions::add_s::add_s;
use crate::functions::lua_call::lua_call;
use crate::functions::lua_gettable::lua_gettable;
use crate::functions::lua_isstring::lua_isstring;
use crate::functions::lua_l_addvalue::lua_l_addvalue;
use crate::functions::lua_l_typename::lua_l_typename;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_toboolean::lua_toboolean;
use crate::functions::push_captures::push_captures;
use crate::functions::push_onecapture::push_onecapture;
use crate::macros::lua_l_error::luaL_error;
use crate::macros::lua_pop::lua_pop;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::records::match_state::MatchState;
use core::ffi::c_char;

pub fn add_value(
    ms: *mut MatchState,
    b: *mut LuaLStrbuf,
    s: *const c_char,
    e: *const c_char,
    tr: i32,
) {
    unsafe {
        let L = (*ms).L;
        if tr == lua_Type::LUA_TFUNCTION as i32 {
            lua_pushvalue(L, 3);
            let n = push_captures(ms, s, e);
            lua_call(L, n, 1);
        } else if tr == lua_Type::LUA_TTABLE as i32 {
            push_onecapture(ms, 0, s, e);
            lua_gettable(L, 3);
        } else {
            // LUA_TNUMBER or LUA_TSTRING
            add_s(ms, b, s, e);
            return;
        }

        if lua_toboolean(L, -1) == 0 {
            // nil or false?
            lua_pop(L, 1);
            lua_pushlstring(L, s, e.offset_from(s) as usize); // keep original text
        } else if lua_isstring(L, -1) == 0 {
            let tn = core::ffi::CStr::from_ptr(lua_l_typename(L, -1)).to_string_lossy();
            luaL_error!(L, "invalid replacement value (a {})", tn);
        }
        lua_l_addvalue(b); // add result to accumulator
    }
}
