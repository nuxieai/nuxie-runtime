//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:99:index2addr`
//! Source: `VM/src/lapi.cpp:99-118` (hand-ported)

use crate::functions::pseudo_2_addr::pseudo_2_addr;
use crate::macros::api_check::api_check;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::lua_registryindex::LUA_REGISTRYINDEX;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

pub unsafe fn index2addr(L: *mut lua_State, idx: core::ffi::c_int) -> StkId {
    if idx > 0 {
        let o = (*L).base.add((idx - 1) as usize);
        api_check!(L, idx as isize <= (*(*L).ci).top.offset_from((*L).base));
        if o >= (*L).top {
            luaO_nilobject as *mut TValue
        } else {
            o
        }
    } else if idx > LUA_REGISTRYINDEX {
        api_check!(
            L,
            idx != 0 && (-idx) as isize <= (*L).top.offset_from((*L).base)
        );
        (*L).top.offset(idx as isize)
    } else {
        pseudo_2_addr(L, idx)
    }
}

#[allow(unused_imports)]
pub use index2addr as index_2_addr;
