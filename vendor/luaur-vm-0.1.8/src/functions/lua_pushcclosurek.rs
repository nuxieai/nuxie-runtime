//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:742:lua_pushcclosurek`
//! Source: `VM/src/lapi.cpp:742-760` (hand-ported)

use core::ffi::{c_char, c_int};

use crate::functions::getcurrenv::getcurrenv;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_f_new_cclosure::luaF_newCclosure;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::iswhite::iswhite;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::setclvalue::setclvalue;
use crate::macros::setobj_2_n::setobj2n;
use crate::records::closure::CClosure;
use crate::records::gc_object::GCObject;
use crate::type_aliases::lua_c_function::LuaCFunction;
use crate::type_aliases::lua_continuation::LuaContinuation;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_pushcclosurek(
    L: *mut lua_State,
    r#fn: LuaCFunction,
    debugname: *const c_char,
    mut nup: c_int,
    cont: LuaContinuation,
) {
    api_check!(L, r#fn.is_some());
    api_check!(L, nup >= 0);
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);
    api_checknelems!(L, nup);

    let cl = luaF_newCclosure(L, nup, getcurrenv(L));
    let cc = core::ptr::addr_of_mut!((*cl).inner.c) as *mut CClosure;
    (*cc).f = r#fn;
    (*cc).cont = cont;
    (*cc).debugname = debugname;

    (*L).top = (*L).top.sub(nup as usize);
    let upvals = core::ptr::addr_of_mut!((*cc).upvals) as *mut TValue;
    while nup > 0 {
        nup -= 1;
        setobj2n!(L, upvals.add(nup as usize), (*L).top.add(nup as usize));
    }

    setclvalue!(L, (*L).top, cl);
    luaur_common::LUAU_ASSERT!(iswhite!(cl as *mut GCObject));
    api_incr_top!(L);
}
