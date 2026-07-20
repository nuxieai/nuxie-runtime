use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_newbuffer::lua_newbuffer;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;
use core::ffi::c_int;

pub fn buffer_fromstring(L: *mut lua_State) -> c_int {
    unsafe {
        let mut len: usize = 0;
        let val = unsafe { lua_l_checklstring(L, 1, &mut len) };

        let data = lua_newbuffer(L, len);
        unsafe {
            core::ptr::copy_nonoverlapping(val as *const u8, data as *mut u8, len);
        }

        1
    }
}
