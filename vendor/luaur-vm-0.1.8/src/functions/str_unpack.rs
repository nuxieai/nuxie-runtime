use crate::enums::k_option::KOption;
use crate::functions::copywithendian::copywithendian;
use crate::functions::getdetails::getdetails;
use crate::functions::initheader::initheader;
use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_checkstack::lua_l_checkstack;
use crate::functions::lua_l_optinteger::lua_l_optinteger;
use crate::functions::lua_pushinteger::lua_pushinteger;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::functions::posrelat::posrelat;
use crate::functions::unpackint::unpackint;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_checkstring::luaL_checkstring;
use crate::records::ftypes::Ftypes;
use crate::records::header::Header;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int, CStr};

pub unsafe fn str_unpack(L: *mut lua_State) -> c_int {
    let mut h = Header::default();
    let mut fmt = luaL_checkstring!(L, 1);

    let mut ld: usize = 0;
    let data = lua_l_checklstring(L, 2, &mut ld);
    let mut pos = posrelat(lua_l_optinteger(L, 3, 1), ld) - 1;
    if pos < 0 {
        pos = 0;
    }

    let mut n = 0;
    luaL_argcheck!(L, pos as usize <= ld, 3, "initial position out of string");
    initheader(L, &mut h);

    while *fmt != 0 {
        let mut size: c_int = 0;
        let mut ntoalign: c_int = 0;
        let opt = getdetails(&mut h, pos as usize, &mut fmt, &mut size, &mut ntoalign);
        luaL_argcheck!(
            L,
            (ntoalign as usize).wrapping_add(size as usize) <= ld - pos as usize,
            2,
            "data string too short"
        );

        pos += ntoalign;
        lua_l_checkstack(L, 2, "too many results");
        n += 1;

        match opt {
            KOption::Kint => {
                let res = unpackint(L, data.add(pos as usize), h.islittle, size, 1);
                lua_pushnumber(L, res as f64);
            }
            KOption::Kuint => {
                let res = unpackint(L, data.add(pos as usize), h.islittle, size, 0) as u64;
                lua_pushnumber(L, res as f64);
            }
            KOption::Kfloat => {
                let mut u = Ftypes { n: 0.0 };
                copywithendian(
                    unsafe { u.buff.as_mut_ptr() },
                    data.add(pos as usize) as *const c_char,
                    size,
                    h.islittle,
                );
                let num = if size as usize == core::mem::size_of::<core::ffi::c_float>() {
                    unsafe { u.f as f64 }
                } else if size as usize == core::mem::size_of::<core::ffi::c_double>() {
                    unsafe { u.d }
                } else {
                    unsafe { u.n }
                };
                lua_pushnumber(L, num);
            }
            KOption::Kchar => {
                lua_pushlstring(L, data.add(pos as usize), size as usize);
            }
            KOption::Kstring => {
                let len = unpackint(L, data.add(pos as usize), h.islittle, size, 0) as usize;
                luaL_argcheck!(
                    L,
                    len <= ld - pos as usize - size as usize,
                    2,
                    "data string too short"
                );
                lua_pushlstring(L, data.add(pos as usize + size as usize), len);
                pos += len as c_int;
            }
            KOption::Kzstr => {
                let len = CStr::from_ptr(data.add(pos as usize)).to_bytes().len();
                luaL_argcheck!(
                    L,
                    pos as usize + len < ld,
                    2,
                    "unfinished string for format 'z'"
                );
                lua_pushlstring(L, data.add(pos as usize), len);
                pos += len as c_int + 1;
            }
            KOption::Kpaddalign | KOption::Kpadding | KOption::Knop => {
                n -= 1;
            }
        }

        pos += size;
    }

    lua_pushinteger(L, pos + 1);
    n + 1
}
