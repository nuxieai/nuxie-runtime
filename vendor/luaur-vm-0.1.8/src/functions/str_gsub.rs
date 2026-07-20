//! Node: `cxx:Function:Luau.VM:VM/src/lstrlib.cpp:831:str_gsub`
//!
//! `string.gsub` — global substitution. Repeatedly match the pattern against the
//! source (up to `max_s` times), append each replacement via `add_value` and the
//! intervening literal text, then push the result string and the substitution
//! count. Honors a leading `^` anchor (single attempt).

use crate::enums::lua_type::lua_Type;
use crate::functions::add_value::add_value;
use crate::functions::lua_l_addchar::lua_l_addchar;
use crate::functions::lua_l_addlstring::lua_l_addlstring;
use crate::functions::lua_l_buffinit::lua_l_buffinit;
use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_optinteger::lua_l_optinteger;
use crate::functions::lua_l_pushresult::lua_l_pushresult;
use crate::functions::lua_pushinteger::lua_pushinteger;
use crate::functions::lua_type::lua_type;
use crate::functions::prepstate::prepstate;
use crate::functions::r#match::match_item;
use crate::functions::reprepstate::reprepstate;
use crate::macros::lua_l_argexpected::luaL_argexpected;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::records::match_state::MatchState;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};

pub unsafe fn str_gsub(L: *mut lua_State) -> c_int {
    let mut srcl: usize = 0;
    let mut lp: usize = 0;
    let mut src = lua_l_checklstring(L, 1, &mut srcl);
    let mut p = lua_l_checklstring(L, 2, &mut lp);
    let tr = lua_type(L, 3);
    let max_s = lua_l_optinteger(L, 4, srcl as c_int + 1);
    let anchor = *p == b'^' as c_char;
    let mut n: c_int = 0;

    let mut ms: MatchState = core::mem::zeroed();
    let mut b: LuaLStrbuf = LuaLStrbuf {
        p: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        L: core::ptr::null_mut(),
        storage: core::ptr::null_mut(),
        buffer: [0; 512],
    };

    luaL_argexpected!(
        L,
        tr == lua_Type::LUA_TNUMBER as c_int
            || tr == lua_Type::LUA_TSTRING as c_int
            || tr == lua_Type::LUA_TFUNCTION as c_int
            || tr == lua_Type::LUA_TTABLE as c_int,
        3,
        "string/function/table"
    );

    lua_l_buffinit(L, &mut b);

    if anchor {
        p = p.add(1);
        lp -= 1; // skip anchor character
    }

    prepstate(&mut ms, L, src, srcl, p, lp);

    while n < max_s {
        reprepstate(&mut ms);
        let e = match_item(&mut ms, src, p);
        if !e.is_null() {
            n += 1;
            add_value(&mut ms, &mut b, src, e, tr);
        }

        if !e.is_null() && e > src {
            // non empty match?
            src = e; // skip it
        } else if src < ms.src_end {
            lua_l_addchar(&mut b, *src);
            src = src.add(1);
        } else {
            break;
        }

        if anchor {
            break;
        }
    }

    lua_l_addlstring(&mut b, src, ms.src_end.offset_from(src) as usize);
    lua_l_pushresult(&mut b);
    lua_pushinteger(L, n); // number of substitutions
    2
}
