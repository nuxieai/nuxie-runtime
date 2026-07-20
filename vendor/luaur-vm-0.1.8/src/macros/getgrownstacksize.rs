use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
#[inline]
pub fn getgrownstacksize(L: *mut lua_State, n: core::ffi::c_int) -> core::ffi::c_int {
    unsafe {
        if n <= (*L).stacksize {
            2 * (*L).stacksize
        } else {
            (*L).stacksize + n
        }
    }
}
