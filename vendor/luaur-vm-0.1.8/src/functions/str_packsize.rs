use crate::enums::k_option::KOption;
use crate::functions::getdetails::getdetails;
use crate::functions::initheader::initheader;
use crate::functions::lua_pushinteger::lua_pushinteger;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_checkstring::luaL_checkstring;
use crate::records::header::Header;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};

const MAXSSIZE: usize = 1 << 30;

#[allow(non_snake_case)]
pub fn str_packsize(l: *mut lua_State) -> c_int {
    let mut h = Header {
        L: core::ptr::null_mut(),
        islittle: 0,
        maxalign: 0,
    };
    let fmt_ptr = unsafe { luaL_checkstring!(l, 1) };
    let mut fmt = fmt_ptr;
    let mut totalsize: usize = 0;

    initheader(l, &mut h);

    while unsafe { *fmt } != b'\0' as c_char {
        let mut size: c_int = 0;
        let mut ntoalign: c_int = 0;
        let mut fmt_cursor = fmt;

        let opt =
            unsafe { getdetails(&mut h, totalsize, &mut fmt_cursor, &mut size, &mut ntoalign) };
        fmt = fmt_cursor;

        luaL_argcheck!(
            l,
            opt != KOption::Kstring && opt != KOption::Kzstr,
            1,
            "variable-length format"
        );

        let total_option_size = (size + ntoalign) as usize;
        luaL_argcheck!(
            l,
            totalsize <= MAXSSIZE - total_option_size,
            1,
            "format result too large"
        );

        totalsize += total_option_size;
    }

    lua_pushinteger(l, totalsize as c_int);
    1
}
