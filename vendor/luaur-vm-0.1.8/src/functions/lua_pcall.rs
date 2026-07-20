use core::ffi::{c_int, c_void};

use crate::functions::f_call::f_call;
use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_d_pcall::luaD_pcall;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::savestack::savestack;
use crate::records::call_s::CallS;
use crate::records::lua_state::lua_State;
use crate::type_aliases::pfunc::Pfunc;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_pcall(L: *mut lua_State, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int {
    api_check!(L, nargs >= 0);
    api_check!(L, nresults >= LUA_MULTRET);
    api_checknelems!(L, nargs + 1);
    api_check!(L, (*L).status == 0);
    api_check!(
        L,
        nresults == LUA_MULTRET
            || (*(*L).ci).top.offset_from((*L).top) >= (nresults - nargs) as isize
    );

    let mut func: isize = 0;
    if errfunc != 0 {
        let o: StkId = index2addr(L, errfunc);
        api_check!(L, o != luaO_nilobject as StkId);
        func = savestack!(L, o) as isize;
    }

    let mut c = CallS {
        func: (*L).top.sub((nargs + 1) as usize),
        nresults,
    };

    let pfunc: Pfunc = Some(f_call);

    let status = luaD_pcall(
        L,
        pfunc,
        &mut c as *mut CallS as *mut c_void,
        savestack!(L, c.func) as isize,
        func,
    );

    if nresults == LUA_MULTRET && (*L).top.offset_from((*(*L).ci).top) >= 0 {
        (*(*L).ci).top = (*L).top;
    }

    status
}
