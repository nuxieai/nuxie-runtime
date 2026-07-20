use core::ffi::{c_int, c_void};

use crate::enums::lua_type::lua_Type;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_u_newudata::lua_u_newudata;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::isblack::isblack;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::records::gc_object::GCObject;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_newuserdatataggedwithmetatable(
    L: *mut lua_State,
    sz: usize,
    tag: c_int,
) -> *mut c_void {
    api_check!(L, (tag as u32) < LUA_UTAG_LIMIT as u32);
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);

    let u = lua_u_newudata(L, sz, tag);

    luaur_common::LUAU_ASSERT!(!isblack!(u as *mut GCObject));

    let h = (*(*L).global).udatamt[tag as usize];
    api_check!(L, !h.is_null());

    (*u).metatable = h;

    (*(*L).top).value.gc = u as *mut GCObject;
    (*(*L).top).tt = lua_Type::LUA_TUSERDATA as c_int;
    crate::macros::checkliveness::checkliveness!((*L).global, (*L).top);
    api_incr_top!(L);

    (*u).data.as_mut_ptr().cast()
}
