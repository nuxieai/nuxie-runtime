use crate::functions::lua_l_checkinteger_64::lua_l_checkinteger_64;
use crate::functions::lua_pushinteger_64::lua_pushinteger_64;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn int64_countrz(L: *mut lua_State) -> core::ffi::c_int {
    let n = lua_l_checkinteger_64(L, 1) as u64;

    // Rust's trailing_zeros() on a u64 is equivalent to __builtin_ctzll on GCC/Clang
    // and the _BitScanForward64 logic on MSVC.
    // For n == 0, trailing_zeros() returns 64, which matches the C++ logic.
    let result = n.trailing_zeros() as i64;

    lua_pushinteger_64(L, result);

    1
}
