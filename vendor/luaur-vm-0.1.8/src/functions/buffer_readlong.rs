use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_pushinteger_64::lua_pushinteger_64;
use crate::macros::isoutofbounds::isoutofbounds;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::functions::buffer_swapbe::buffer_swapbe;

#[allow(non_snake_case)]
pub unsafe fn buffer_readlong(L: *mut lua_State) -> core::ffi::c_int {
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len);
    let offset = lua_l_checkinteger(L, 2);

    if isoutofbounds(offset, len, core::mem::size_of::<u64>()) {
        luaL_error!(L, "buffer access out of bounds");
    }

    let mut val: u64 = 0;
    core::ptr::copy_nonoverlapping(
        (buf as *const u8).add(offset as usize),
        &mut val as *mut u64 as *mut u8,
        core::mem::size_of::<u64>(),
    );

    if LUAU_BIG_ENDIAN {
        val = buffer_swapbe(val);
    }

    lua_pushinteger_64(L, val as i64);
    1
}
