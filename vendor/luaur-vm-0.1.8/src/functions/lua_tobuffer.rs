use core::ffi::{c_int, c_void};

use crate::functions::index_2_addr::index2addr;
use crate::macros::bufvalue::bufvalue;
use crate::macros::ttisbuffer::ttisbuffer;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_tobuffer(L: *mut lua_State, idx: c_int, len: *mut usize) -> *mut c_void {
    let o: StkId = index2addr(L, idx);

    if !ttisbuffer!(o) {
        return core::ptr::null_mut();
    }

    let b = bufvalue!(o);

    if !len.is_null() {
        *len = (*b).len as usize;
    }

    (*b).data.as_ptr() as *mut c_void
}
