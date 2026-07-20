use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

const NB: i32 = 8;
const MC: i32 = 0xff;
const SZINT: i32 = core::mem::size_of::<core::ffi::c_longlong>() as i32;

pub fn unpackint(
    L: *mut lua_State,
    str: *const c_char,
    islittle: i32,
    size: i32,
    issigned: i32,
) -> i64 {
    let mut res: u64 = 0;
    let mut i: i32 = 0;
    let limit = if size <= SZINT { size } else { SZINT };

    i = limit - 1;
    while i >= 0 {
        res <<= NB as u32;
        let idx = if islittle != 0 { i } else { size - 1 - i };
        let byte = unsafe { *str.offset(idx as isize) as u8 };
        res |= byte as u64;
        i -= 1;
    }

    if size < SZINT {
        if issigned != 0 {
            let mask = 1u64 << ((size * NB) - 1);
            // C does `(res ^ mask) - mask` in unsigned (wrapping) arithmetic for sign extension.
            res = (res ^ mask).wrapping_sub(mask);
        }
    } else if size > SZINT {
        let mask = if issigned == 0 || res as i64 >= 0 {
            0
        } else {
            MC
        };
        i = limit;
        while i < size {
            let idx = if islittle != 0 { i } else { size - 1 - i };
            let byte = unsafe { *str.offset(idx as isize) as u8 };
            if byte as i32 != mask {
                unsafe {
                    luaL_error!(L, "{}-byte integer does not fit into Lua Integer", size);
                }
            }
            i += 1;
        }
    }

    res as i64
}
