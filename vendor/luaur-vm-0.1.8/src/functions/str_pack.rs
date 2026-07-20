//! Node: `cxx:Function:Luau.VM:VM/src/lstrlib.cpp:1407:str_pack`

use crate::enums::k_option::KOption;
use crate::functions::copywithendian::copywithendian;
use crate::functions::getdetails::getdetails;
use crate::functions::initheader::initheader;
use crate::functions::lua_l_buffinit::lua_l_buffinit;
use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::functions::lua_l_pushresult::lua_l_pushresult;
use crate::functions::packint::packint;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_checkstring::luaL_checkstring;
use crate::records::ftypes::Ftypes;
use crate::records::header::Header;
use crate::records::lua_l_strbuf::{LuaLStrbuf, LUA_BUFFERSIZE};
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};
use core::ptr::null_mut;

pub const LUAL_PACKPADBYTE: u8 = 0x00;

pub unsafe fn str_pack(L: *mut lua_State) -> c_int {
    let mut b = LuaLStrbuf {
        p: null_mut(),
        end: null_mut(),
        L: null_mut(),
        storage: null_mut(),
        buffer: [0; LUA_BUFFERSIZE],
    };
    let mut h = Header::default();
    let mut fmt = luaL_checkstring!(L, 1); // format string
    let mut arg = 1; // current argument to pack
    let mut totalsize: usize = 0; // accumulate total size of result
    initheader(L, &mut h);
    crate::functions::lua_pushnil::lua_pushnil(L); // mark to separate arguments from string buffer
    lua_l_buffinit(L, &mut b);
    while *fmt != 0 {
        let mut size: c_int = 0;
        let mut ntoalign: c_int = 0;
        let opt = getdetails(&mut h, totalsize, &mut fmt, &mut size, &mut ntoalign);
        totalsize += (ntoalign + size) as usize;
        while ntoalign > 0 {
            ntoalign -= 1;
            crate::functions::lua_l_addchar::lua_l_addchar(&mut b, LUAL_PACKPADBYTE as c_char);
            // fill alignment
        }
        arg += 1;
        match opt {
            KOption::Kint => {
                // signed integers
                let n = lua_l_checknumber(L, arg) as i64;
                if size < core::mem::size_of::<i64>() as c_int {
                    // need overflow check?
                    let lim = 1i64 << ((size * 8) - 1);
                    luaL_argcheck!(L, -lim <= n && n < lim, arg, "integer overflow");
                }
                packint(&mut b, n as u64, h.islittle, size, (n < 0) as i32);
            }
            KOption::Kuint => {
                // unsigned integers
                let n = lua_l_checknumber(L, arg) as i64;
                if size < core::mem::size_of::<i64>() as c_int {
                    // need overflow check?
                    luaL_argcheck!(
                        L,
                        (n as u64) < (1u64 << (size * 8)),
                        arg,
                        "unsigned overflow"
                    );
                }
                packint(&mut b, n as u64, h.islittle, size, 0);
            }
            KOption::Kfloat => {
                // floating-point options
                let mut u = Ftypes { n: 0.0 };
                let mut buff = [0i8; 16]; // MAXINTSIZE
                let n = lua_l_checknumber(L, arg); // get argument
                if size as usize == core::mem::size_of::<core::ffi::c_float>() {
                    u.f = n as core::ffi::c_float; // copy it into 'u'
                } else if size as usize == core::mem::size_of::<core::ffi::c_double>() {
                    u.d = n;
                } else {
                    u.n = n;
                }
                // move 'u' to final result, correcting endianness if needed
                copywithendian(buff.as_mut_ptr(), u.buff.as_ptr(), size, h.islittle);
                crate::functions::lua_l_addlstring::lua_l_addlstring(
                    &mut b,
                    buff.as_ptr(),
                    size as usize,
                );
            }
            KOption::Kchar => {
                // fixed-size string
                let mut len: usize = 0;
                let s = lua_l_checklstring(L, arg, &mut len);
                luaL_argcheck!(
                    L,
                    len <= size as usize,
                    arg,
                    "string longer than given size"
                );
                crate::functions::lua_l_addlstring::lua_l_addlstring(&mut b, s, len); // add string
                while len < size as usize {
                    len += 1;
                    crate::functions::lua_l_addchar::lua_l_addchar(
                        &mut b,
                        LUAL_PACKPADBYTE as c_char,
                    );
                }
            }
            KOption::Kstring => {
                // strings with length count
                let mut len: usize = 0;
                let s = lua_l_checklstring(L, arg, &mut len);
                luaL_argcheck!(
                    L,
                    size >= core::mem::size_of::<usize>() as c_int || len < (1usize << (size * 8)),
                    arg,
                    "string length does not fit in given size"
                );
                packint(&mut b, len as u64, h.islittle, size, 0); // pack length
                crate::functions::lua_l_addlstring::lua_l_addlstring(&mut b, s, len);
                totalsize += len;
            }
            KOption::Kzstr => {
                // zero-terminated string
                let mut len: usize = 0;
                let s = lua_l_checklstring(L, arg, &mut len);
                luaL_argcheck!(
                    L,
                    core::ffi::CStr::from_ptr(s).to_bytes().len() == len,
                    arg,
                    "string contains zeros"
                );
                crate::functions::lua_l_addlstring::lua_l_addlstring(&mut b, s, len);
                crate::functions::lua_l_addchar::lua_l_addchar(&mut b, 0); // add zero at the end
                totalsize += len + 1;
            }
            KOption::Kpadding => {
                crate::functions::lua_l_addchar::lua_l_addchar(&mut b, LUAL_PACKPADBYTE as c_char);
                arg -= 1; // undo increment
            }
            KOption::Kpaddalign | KOption::Knop => {
                arg -= 1; // undo increment
            }
        }
    }
    lua_l_pushresult(&mut b);
    1
}
