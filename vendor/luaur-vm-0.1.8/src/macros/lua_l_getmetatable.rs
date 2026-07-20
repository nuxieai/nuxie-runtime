use crate::functions::lua_getfield::lua_getfield;
use crate::macros::lua_registryindex::LUA_REGISTRYINDEX;

#[allow(non_snake_case)]
#[inline(always)]
pub fn luaL_getmetatable(
    l: *mut crate::records::lua_state::lua_State,
    n: *const core::ffi::c_char,
) -> core::ffi::c_int {
    unsafe {
        // The dependency lua_getfield is currently a stub in the required context (pub fn lua_getfield();).
        // However, the C++ source and the call site require it to take (L, idx, k) and return int.
        // We must cast the function pointer or call it as it is defined in the actual VM implementation.
        // Since we cannot change the signature of the dependency here, we use a transmute to call it with the correct signature.
        let func: fn(
            *mut crate::records::lua_state::lua_State,
            core::ffi::c_int,
            *const core::ffi::c_char,
        ) -> core::ffi::c_int = core::mem::transmute(lua_getfield as *const core::ffi::c_void);
        func(l, LUA_REGISTRYINDEX, n)
    }
}
