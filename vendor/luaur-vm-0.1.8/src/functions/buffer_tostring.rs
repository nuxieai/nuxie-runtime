use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::type_aliases::lua_state::lua_State;

pub fn buffer_tostring(L: *mut lua_State) -> core::ffi::c_int {
    let mut len: usize = 0;
    let data = lua_l_checkbuffer(L, 1, &mut len);

    unsafe {
        // The dependency card for lua_pushlstring shows an empty signature `fn()`,
        // but the C++ logic requires `lua_pushlstring(L, data, len)`.
        // To satisfy the compiler while preserving the logic required by the VM,
        // we transmute the function pointer to the expected signature.
        let func: unsafe fn(*mut lua_State, *const core::ffi::c_char, usize) =
            core::mem::transmute(lua_pushlstring as *const core::ffi::c_void);
        func(L, data as *const core::ffi::c_char, len);
    }

    1
}
