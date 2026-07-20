#[allow(non_snake_case)]
#[inline(always)]
pub const fn lua_ispseudo(i: core::ffi::c_int) -> bool {
    i <= crate::macros::lua_registryindex::LUA_REGISTRYINDEX
}
