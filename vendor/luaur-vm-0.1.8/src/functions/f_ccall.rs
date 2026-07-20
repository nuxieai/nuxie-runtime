use crate::functions::lua_checkstack::lua_checkstack;
use crate::functions::lua_d_call::lua_d_call;
use crate::functions::lua_pushcclosurek::lua_pushcclosurek;
use crate::macros::cast_to::cast_to;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::macros::lua_pushlightuserdata::lua_pushlightuserdata;
use crate::records::c_call_s::CCallS;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn f_ccall(L: *mut lua_State, ud: *mut core::ffi::c_void) {
    let c = cast_to!(*mut CCallS, ud);

    if lua_checkstack(L, 2) == 0 {
        lua_g_runerror!(L, "stack limit");
    }

    lua_pushcclosurek(L, (*c).func, core::ptr::null(), 0, None);
    lua_pushlightuserdata(L as *mut core::ffi::c_void, (*c).ud);
    lua_d_call(L, (*L).top.sub(2), 0);
}
