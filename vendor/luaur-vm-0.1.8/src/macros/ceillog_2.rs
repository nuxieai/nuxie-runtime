use crate::functions::lua_o_log_2::luaO_log2;

#[allow(non_snake_case)]
pub fn ceillog2(x: core::ffi::c_uint) -> core::ffi::c_int {
    luaO_log2(x.wrapping_sub(1)) + 1
}
