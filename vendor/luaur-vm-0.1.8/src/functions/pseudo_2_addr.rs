//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:73:pseudo2addr`
//! Source: `VM/src/lapi.cpp:65-97` (hand-ported; includes the file-static
//! `getcurrenv` helper inlined here since it was never a graph node)

use crate::macros::api_check::api_check;
use crate::macros::curr_func::curr_func;
use crate::macros::lua_environindex::LUA_ENVIRONINDEX;
use crate::macros::lua_globalsindex::LUA_GLOBALSINDEX;
use crate::macros::lua_ispseudo::lua_ispseudo;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::lua_registryindex::LUA_REGISTRYINDEX;
use crate::macros::registry::registry;
use crate::macros::sethvalue::sethvalue;
use crate::records::closure::Closure;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

unsafe fn getcurrenv(L: *mut lua_State) -> *mut LuaTable {
    if (*L).ci == (*L).base_ci {
        // no enclosing function? use global table as environment
        (*L).gt
    } else {
        let func = curr_func!(L);
        (*func).env
    }
}

pub unsafe fn pseudo_2_addr(L: *mut lua_State, idx: core::ffi::c_int) -> StkId {
    api_check!(L, lua_ispseudo(idx));
    match idx {
        // pseudo-indices
        LUA_REGISTRYINDEX => registry!(L) as *const TValue as *mut TValue,
        LUA_ENVIRONINDEX => {
            let tmp = &mut (*(*L).global).pseudotemp as *mut TValue;
            sethvalue!(L, tmp, getcurrenv(L));
            tmp
        }
        LUA_GLOBALSINDEX => {
            let tmp = &mut (*(*L).global).pseudotemp as *mut TValue;
            sethvalue!(L, tmp, (*L).gt);
            tmp
        }
        _ => {
            let func = curr_func!(L);
            let idx = LUA_GLOBALSINDEX - idx;
            if idx <= (*func).nupvalues as i32 {
                let c = &mut (*func).inner.c;
                c.upvals.as_mut_ptr().add((idx - 1) as usize)
            } else {
                luaO_nilobject as *mut TValue
            }
        }
    }
}
