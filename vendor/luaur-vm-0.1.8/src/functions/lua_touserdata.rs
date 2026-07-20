use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::ttislightuserdata::ttislightuserdata;
use crate::macros::ttisuserdata::ttisuserdata;
use crate::macros::uvalue::uvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_touserdata(L: *mut lua_State, idx: c_int) -> *mut core::ffi::c_void {
    let o: StkId = index2addr(L, idx);

    if ttisuserdata!(o) {
        // uvalue(o) returns a pointer to the Udata struct.
        // The data field is a char[1] at the end of the struct.
        // We return the address of that array as a void pointer.
        uvalue!(o).data.as_ptr() as *mut core::ffi::c_void
    } else if ttislightuserdata!(o) {
        // pvalue(o) is defined as a constant in the provided context, but the C++ macro
        // accesses (o)->value.p. Based on the provided PVALUE constant and the
        // requirement to follow the C++ logic, we access the pointer value.
        (*o).value.p
    } else {
        core::ptr::null_mut()
    }
}
