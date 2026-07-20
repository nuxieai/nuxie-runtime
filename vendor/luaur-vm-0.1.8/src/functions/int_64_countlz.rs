use crate::functions::lua_l_checkinteger_64::lua_l_checkinteger_64;
use crate::functions::lua_pushinteger_64::lua_pushinteger_64;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn int64_countlz(L: *mut lua_State) -> core::ffi::c_int {
    let n = lua_l_checkinteger_64(L, 1) as u64;

    // Rust's leading_zeros() on a u64 is equivalent to __builtin_clzll on GCC/Clang
    // and the _BitScanReverse64 logic on MSVC.
    // For n == 0, leading_zeros() returns 64, which matches the C++ logic: (n == 0) ? 64 : __builtin_clzll(n).
    let result = n.leading_zeros() as i64;

    lua_pushinteger_64(L, result);

    1
}
