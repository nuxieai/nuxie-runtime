//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:272:lua_insert`
//! Source: `VM/src/lapi.cpp:272-280` (hand-ported)

use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_check::api_check;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::setobj_2_s::setobj2s;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_insert(L: *mut lua_State, idx: c_int) {
    lua_c_threadbarrier_lapi(L);
    let p: StkId = index2addr(L, idx);
    api_check!(L, p != luaO_nilobject as StkId);

    let mut q = (*L).top;
    while q > p {
        setobj2s!(L, q, q.sub(1));
        q = q.sub(1);
    }
    setobj2s!(L, p, (*L).top);
}
