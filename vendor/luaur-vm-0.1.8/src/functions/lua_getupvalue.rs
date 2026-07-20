use core::ffi::c_char;
use core::ffi::c_int;

use crate::functions::aux_upvalue::aux_upvalue;
use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::setobj_2_s::setobj2s;
use crate::records::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_getupvalue(L: *mut lua_State, funcindex: c_int, n: c_int) -> *const c_char {
    lua_c_threadbarrier_lapi(L);
    let mut val: *mut TValue = core::ptr::null_mut();
    let name: *const c_char = aux_upvalue(index2addr(L, funcindex), n, &mut val);

    if !name.is_null() {
        setobj2s!(L, (*L).top, val);
        api_incr_top!(L);
    }

    name
}
