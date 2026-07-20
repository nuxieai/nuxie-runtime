//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:1082:lua_call`
//! Source: `VM/src/lapi.cpp:1082-1092` (hand-ported)

use core::ffi::c_int;

use crate::functions::lua_d_call::lua_d_call;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_call(L: *mut lua_State, nargs: c_int, nresults: c_int) {
    api_check!(L, nargs >= 0);
    api_check!(L, nresults >= LUA_MULTRET);
    api_checknelems!(L, nargs + 1);
    api_check!(L, (*L).status == 0);
    api_check!(
        L,
        nresults == LUA_MULTRET
            || (*(*L).ci).top.offset_from((*L).top) >= (nresults - nargs) as isize
    );

    let func: StkId = (*L).top.sub((nargs + 1) as usize);
    lua_d_call(L, func, nresults);

    if nresults == LUA_MULTRET && (*L).top >= (*(*L).ci).top {
        (*(*L).ci).top = (*L).top;
    }
}
