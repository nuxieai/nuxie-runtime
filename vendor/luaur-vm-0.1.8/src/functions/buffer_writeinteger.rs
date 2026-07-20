use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::macros::isoutofbounds::isoutofbounds;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;

use crate::functions::buffer_swapbe::buffer_swapbe;
use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;

#[allow(non_snake_case)]
pub unsafe fn buffer_writeinteger<T>(L: *mut lua_State) -> core::ffi::c_int
where
    T: Copy,
{
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len) as *mut core::ffi::c_char;
    let offset = lua_l_checkinteger(L, 2);
    let value = lua_l_checkunsigned(L, 3);

    if isoutofbounds(offset, len, core::mem::size_of::<T>()) {
        luaL_error!(L, "buffer access out of bounds");
    }

    let mut val: T = core::mem::transmute_copy::<u32, T>(&value);

    if LUAU_BIG_ENDIAN {
        val = buffer_swapbe(val);
    }

    core::ptr::copy_nonoverlapping(
        &val as *const T as *const u8,
        (buf as *mut u8).add(offset as usize),
        core::mem::size_of::<T>(),
    );
    0
}
