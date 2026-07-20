use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;

pub fn buffer_readbits(L: *mut lua_State) -> core::ffi::c_int {
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len) as *mut core::ffi::c_char;
    let bitoffset = lua_l_checknumber(L, 2) as i64;
    let bitcount = lua_l_checkinteger(L, 3);

    if bitoffset < 0 {
        luaL_error!(L, "buffer access out of bounds");
    }

    if (bitcount as u32) > 32 {
        luaL_error!(L, "bit count is out of range of [0; 32]");
    }

    if (bitoffset as u64 + bitcount as u64) > (len as u64) * 8 {
        luaL_error!(L, "buffer access out of bounds");
    }

    let startbyte = (bitoffset / 8) as u32;
    let endbyte = ((bitoffset + bitcount as i64 + 7) / 8) as u32;

    let mut data: u64 = 0;

    if LUAU_BIG_ENDIAN {
        for i in (startbyte as i32..=endbyte as i32 - 1).rev() {
            data = (data << 8) + (unsafe { *buf.add(i as usize) } as u8) as u64;
        }
    } else {
        let src = unsafe { buf.add(startbyte as usize) };
        let dst = &mut data as *mut u64 as *mut core::ffi::c_char;
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, (endbyte - startbyte) as usize);
        }
    }

    let subbyteoffset = (bitoffset & 0x7) as u64;
    let mask = (1u64 << bitcount as u64) - 1;

    let result = ((data >> subbyteoffset) & mask) as u32;
    crate::functions::lua_pushunsigned::lua_pushunsigned(L, result);

    1
}
