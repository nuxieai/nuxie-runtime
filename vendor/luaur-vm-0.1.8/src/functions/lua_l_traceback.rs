use core::ffi::{c_char, c_int, CStr};
use std::ffi::CString;

use crate::functions::lua_getinfo::lua_getinfo;
use crate::functions::lua_l_addchar::lua_l_addchar;
use crate::functions::lua_l_addlstring::lua_l_addlstring;
use crate::functions::lua_l_addstring::lua_l_addstring;
use crate::functions::lua_l_buffinit::lua_l_buffinit;
use crate::functions::lua_l_pushresult::lua_l_pushresult;
use crate::records::lua_debug::LuaDebug;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::type_aliases::lua_state::lua_State;

/// Build a traceback string from `L1`, optionally prepending `msg`, and push
/// the result onto `L`. Faithful 1:1 port of `luaL_traceback` from
/// `luau/VM/src/laux.cpp:381-425`.
#[allow(non_snake_case)]
pub unsafe fn lua_l_traceback(
    L: *mut lua_State,
    L1: *mut lua_State,
    msg: Option<&str>,
    level: c_int,
) {
    debug_assert!(level >= 0);

    // Helper: manually convert an int to decimal and append to the buffer.
    unsafe fn addsignednum(buf: *mut LuaLStrbuf, n: i32) {
        let mut line = [0 as c_char; 32];
        let lineend = line.len();
        let mut lineptr = lineend;
        let mut r = n as u32;
        while r > 0 {
            lineptr -= 1;
            line[lineptr] = (b'0' + (r % 10) as u8) as c_char;
            r /= 10;
        }

        lua_l_addlstring(buf, line.as_ptr().add(lineptr), lineend - lineptr);
    }

    let mut buf = LuaLStrbuf {
        p: core::ptr::null_mut(),
        end: core::ptr::null_mut(),
        L: core::ptr::null_mut(),
        storage: core::ptr::null_mut(),
        buffer: [0; 512],
    };
    lua_l_buffinit(L, &mut buf);

    if let Some(msg_str) = msg {
        let c_msg = CString::new(msg_str).unwrap_or_default();
        lua_l_addstring(&mut buf, c_msg.as_ptr());
        lua_l_addstring(&mut buf, c"\n".as_ptr());
    }

    let mut ar: LuaDebug = core::mem::zeroed();
    let mut i: c_int = level;

    while lua_getinfo(L1, i, c"sln".as_ptr(), &mut ar) != 0 {
        if CStr::from_ptr(ar.what).to_bytes() == b"C" {
            i += 1;
            continue;
        }

        if !ar.source.is_null() {
            lua_l_addstring(&mut buf, ar.short_src);
        }

        if ar.currentline > 0 {
            lua_l_addchar(&mut buf, b':' as c_char);
            addsignednum(&mut buf, ar.currentline);
        }

        if !ar.name.is_null() {
            lua_l_addstring(&mut buf, c" function ".as_ptr());
            lua_l_addstring(&mut buf, ar.name);
        }

        lua_l_addchar(&mut buf, b'\n' as c_char);

        i += 1;
    }

    lua_l_pushresult(&mut buf);
}
