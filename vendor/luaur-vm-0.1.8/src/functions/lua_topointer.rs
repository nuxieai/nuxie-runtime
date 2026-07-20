use core::ffi::c_int;
use core::ffi::c_void;

use crate::enums::lua_type::lua_Type;
use crate::functions::index_2_addr::index2addr;
use crate::macros::gcvalue::gcvalue;
use crate::macros::iscollectable::iscollectable;
use crate::macros::ttype::ttype;
use crate::macros::uvalue::uvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_topointer(L: *mut lua_State, idx: c_int) -> *const c_void {
    let o: StkId = index2addr(L, idx);
    let tt = ttype!(o);

    if tt == lua_Type::LUA_TUSERDATA as i32 {
        (*uvalue!(o)).data.as_ptr() as *const c_void
    } else if tt == lua_Type::LUA_TLIGHTUSERDATA as i32 {
        // pvalue(o) is defined as check_exp(ttislightuserdata(o), (o)->value.p)
        // In Rust, the pvalue macro is currently a stub or constant,
        // so we access the union field directly to match the C++ logic.
        (*o).value.p as *const c_void
    } else {
        if iscollectable!(o) {
            gcvalue!(o) as *const c_void
        } else {
            core::ptr::null()
        }
    }
}
