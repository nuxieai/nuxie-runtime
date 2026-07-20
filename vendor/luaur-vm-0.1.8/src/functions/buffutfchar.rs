use crate::functions::lua_l_checkinteger::luaL_checkinteger;
use crate::functions::lua_o_utf_8_esc::lua_o_utf_8_esc;
use crate::macros::cast_to::cast_to;

use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::type_aliases::lua_state::lua_State;

use core::ffi::{c_char, c_int};

const MAXUNICODE: c_int = 0x10FFFF;
const UTF8BUFFSZ: usize = 8;

pub fn buffutfchar(
    l: *mut lua_State,
    arg: c_int,
    buff: *mut c_char,
    charstr: *mut *const c_char,
) -> c_int {
    let code = luaL_checkinteger(l, arg);
    luaL_argcheck!(
        l,
        0 <= code && code <= MAXUNICODE,
        arg,
        "value out of range"
    );

    let buff_slice = unsafe { core::slice::from_raw_parts_mut(buff, UTF8BUFFSZ) };
    let lval = lua_o_utf_8_esc(
        buff_slice.try_into().expect("UTF8BUFFSZ mismatch"),
        cast_to!(i64, code) as u32,
    );

    unsafe {
        *charstr = buff.add(UTF8BUFFSZ).wrapping_sub(lval as usize) as *const c_char;
    }

    lval
}
