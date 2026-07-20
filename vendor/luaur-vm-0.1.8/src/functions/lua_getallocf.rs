use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_alloc::lua_Alloc;

pub fn lua_getallocf(L: *mut lua_State, ud: *mut *mut core::ffi::c_void) -> lua_Alloc {
    let f = unsafe { (*(*L).global).frealloc };
    if !ud.is_null() {
        unsafe { *ud = (*(*L).global).ud };
    }
    f
}
