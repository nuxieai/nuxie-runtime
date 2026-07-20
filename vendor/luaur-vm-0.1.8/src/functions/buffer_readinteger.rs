use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::functions::buffer_swapbe::buffer_swapbe;
use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::macros::isoutofbounds::isoutofbounds;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn buffer_readinteger<T>(L: *mut lua_State) -> core::ffi::c_int
where
    T: Copy + Into<f64>,
{
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len);
    let offset = lua_l_checkinteger(L, 2);

    if isoutofbounds(offset, len, core::mem::size_of::<T>()) {
        luaL_error!(L, "buffer access out of bounds");
    }

    let mut val: T = core::mem::zeroed();
    core::ptr::copy_nonoverlapping(
        (buf as *const u8).add(offset as usize),
        &mut val as *mut T as *mut u8,
        core::mem::size_of::<T>(),
    );

    if LUAU_BIG_ENDIAN {
        val = buffer_swapbe(val);
    }

    lua_pushnumber(L, val.into());
    1
}
