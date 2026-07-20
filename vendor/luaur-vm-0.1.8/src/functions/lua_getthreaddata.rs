use crate::records::lua_state::lua_State;
use core::ffi::c_void;

#[inline]
pub fn lua_getthreaddata(l: *mut lua_State) -> *mut c_void {
    unsafe { (*l).userdata }
}
