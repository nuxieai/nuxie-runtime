use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;

pub fn buffer_writebits(L: *mut lua_State) -> core::ffi::c_int {
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len) as *mut core::ffi::c_char;
    let bitoffset = lua_l_checknumber(L, 2) as i64;
    let bitcount = lua_l_checkinteger(L, 3);
    let value = lua_l_checkunsigned(L, 4);

    if bitoffset < 0 {
        unsafe {
            luaL_error!(L, "buffer access out of bounds");
        }
    }

    if bitcount < 0 || bitcount > 32 {
        unsafe {
            luaL_error!(L, "bit count is out of range of [0; 32]");
        }
    }

    if (bitoffset as u64 + bitcount as u64) > (len as u64) * 8 {
        unsafe {
            luaL_error!(L, "buffer access out of bounds");
        }
    }

    let startbyte = (bitoffset / 8) as usize;
    let endbyte = ((bitoffset + bitcount as i64 + 7) / 8) as usize;

    let mut data: u64 = 0;

    if LUAU_BIG_ENDIAN {
        for i in (startbyte..endbyte).rev() {
            data = data * 256 + (unsafe { *buf.add(i) } as u8) as u64;
        }
    } else {
        unsafe {
            core::ptr::copy_nonoverlapping(
                buf.add(startbyte),
                &mut data as *mut u64 as *mut core::ffi::c_char,
                endbyte - startbyte,
            );
        }
    }

    let subbyteoffset = (bitoffset & 0x7) as u64;
    let mask = (((1u64 << bitcount) - 1) << subbyteoffset) as u64;

    data = (data & !mask) | (((value as u64) << subbyteoffset) & mask);

    if LUAU_BIG_ENDIAN {
        for i in startbyte..endbyte {
            unsafe {
                *buf.add(i) = (data & 0xff) as core::ffi::c_char;
            }
            data >>= 8;
        }
    } else {
        unsafe {
            core::ptr::copy_nonoverlapping(
                &data as *const u64 as *const core::ffi::c_char,
                buf.add(startbyte),
                endbyte - startbyte,
            );
        }
    }

    0
}
