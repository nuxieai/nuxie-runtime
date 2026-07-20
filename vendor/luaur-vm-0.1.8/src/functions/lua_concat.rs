use core::ffi::c_int;

use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::functions::lua_v_concat::lua_v_concat;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::cast_int::cast_int;
use crate::macros::isblack::isblack;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::setsvalue::setsvalue;
use crate::records::gc_object::GCObject;
use crate::records::lua_state::lua_State;

pub(crate) unsafe fn lua_c_threadbarrier_lapi(L: *mut lua_State) {
    let obj = L as *mut GCObject;
    if isblack!(obj) {
        lua_c_barrierback(L, obj, &mut (*L).gclist);
    }
}

#[allow(non_snake_case)]
pub unsafe fn lua_concat(L: *mut lua_State, n: c_int) {
    api_check!(L, n >= 0);
    api_checknelems!(L, n);

    if n >= 2 {
        luaC_checkGC!(L);
        lua_c_threadbarrier_lapi(L);
        lua_v_concat(L, n, cast_int!((*L).top.offset_from((*L).base)) - 1);
        (*L).top = (*L).top.sub((n - 1) as usize);
    } else if n == 0 {
        lua_c_threadbarrier_lapi(L);
        setsvalue!(L, (*L).top, luaS_newlstr(L, c"".as_ptr(), 0));
        api_incr_top!(L);
    }
}
