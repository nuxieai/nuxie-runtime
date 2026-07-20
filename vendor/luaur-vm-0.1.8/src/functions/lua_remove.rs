use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::api_check::api_check;
use crate::macros::setobj_2_s::setobj2s;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_remove(L: *mut lua_State, idx: c_int) {
    let p: StkId = index2addr(L, idx);

    // api_checkvalidindex macro expands to a reference to crate::records::lobject::luaO_nilobject,
    // but that module isn't available in this crate layout. Replicate its behavior directly.
    api_check!(L, p != core::ptr::null_mut());

    let mut p = p;
    while p.offset(1) < (*L).top {
        p = p.offset(1);
        setobj2s!(L, p.offset(-1), p);
    }
    (*L).top = (*L).top.offset(-1);
}
