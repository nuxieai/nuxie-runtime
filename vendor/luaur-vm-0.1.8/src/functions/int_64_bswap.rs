use crate::functions::lua_l_checkinteger_64::lua_l_checkinteger_64;
use crate::functions::lua_pushinteger_64::lua_pushinteger_64;
use crate::type_aliases::lua_state::LuaState;

pub unsafe fn int64_bswap(l: *mut LuaState) -> core::ffi::c_int {
    let a = lua_l_checkinteger_64(l, 1) as u64;

    let swapped = (a >> 56)
        | ((a & 0x00FF000000000000) >> 40)
        | ((a & 0x0000FF0000000000) >> 24)
        | ((a & 0x000000FF00000000) >> 8)
        | ((a & 0x00000000FF000000) << 8)
        | ((a & 0x0000000000FF0000) << 24)
        | ((a & 0x000000000000FF00) << 40)
        | ((a & 0x00000000000000FF) << 56);

    lua_pushinteger_64(l, swapped as i64);

    1
}
