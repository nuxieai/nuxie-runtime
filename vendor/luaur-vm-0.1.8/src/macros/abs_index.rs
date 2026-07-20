#[allow(non_snake_case)]
#[inline(always)]
pub fn abs_index(
    L: *mut crate::type_aliases::lua_state::lua_State,
    i: core::ffi::c_int,
) -> core::ffi::c_int {
    if i > 0 || i <= crate::macros::lua_registryindex::LUA_REGISTRYINDEX {
        i
    } else {
        unsafe { crate::functions::lua_gettop::lua_gettop(L) + i + 1 }
    }
}
