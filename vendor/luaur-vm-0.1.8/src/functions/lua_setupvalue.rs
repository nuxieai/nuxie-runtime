use core::ffi::c_char;
use core::ffi::c_int;

use crate::functions::aux_upvalue::aux_upvalue;
use crate::functions::index_2_addr::index_2_addr;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::clvalue::clvalue;
use crate::macros::lua_c_barrier::luaC_barrier;
use crate::macros::setobj::setobj;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_setupvalue(L: *mut lua_State, funcindex: c_int, n: c_int) -> *const c_char {
    api_checknelems!(L, 1);
    let fi: StkId = index_2_addr(L, funcindex);
    let mut val: *mut TValue = core::ptr::null_mut();
    let name: *const c_char = aux_upvalue(fi, n, &mut val);

    if !name.is_null() {
        (*L).top = (*L).top.offset(-1);
        setobj!(L, val, (*L).top);
        luaC_barrier!(L, fi as *mut crate::records::gc_object::GCObject, (*L).top);
    }

    name
}
