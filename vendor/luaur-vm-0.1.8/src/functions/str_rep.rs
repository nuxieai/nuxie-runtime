//! Node: `cxx:Function:Luau.VM:VM/src/lstrlib.cpp:83:str_rep`
//!
//! `string.rep` — repeat the argument `n` times into a single buffer, doubling
//! the already-written prefix each step so the fill is O(result) with log(n)
//! memcpys (the classic exponential-pattern trick), with an overflow guard.

use crate::functions::lua_l_buffinitsize::lua_l_buffinitsize;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_pushresultsize::lua_l_pushresultsize;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::macros::lua_l_error::luaL_error;
use crate::macros::maxssize::MAXSSIZE;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub fn str_rep(L: *mut lua_State) -> c_int {
    unsafe {
        let mut l: usize = 0;
        let s = lua_l_checklstring(L, 1, &mut l);
        let n = lua_l_checkinteger(L, 2);

        if n <= 0 {
            lua_pushlstring(L, c"".as_ptr(), 0);
            return 1;
        }

        if l > (MAXSSIZE as usize) / (n as usize) {
            luaL_error!(L, "resulting string too large");
        }

        let total = l * (n as usize);

        let mut b: LuaLStrbuf = LuaLStrbuf {
            p: core::ptr::null_mut(),
            end: core::ptr::null_mut(),
            L: core::ptr::null_mut(),
            storage: core::ptr::null_mut(),
            buffer: [0; 512],
        };
        let mut ptr = lua_l_buffinitsize(L, &mut b, total);

        let start = ptr;
        let mut left = total;
        let mut step = l;

        core::ptr::copy_nonoverlapping(s, ptr, l);
        ptr = ptr.add(l);
        left -= l;

        // use the increasing 'pattern' inside our target buffer to fill the next part
        while step < left {
            core::ptr::copy_nonoverlapping(start, ptr, step);
            ptr = ptr.add(step);
            left -= step;
            step <<= 1;
        }

        // fill tail
        core::ptr::copy_nonoverlapping(start, ptr, left);

        lua_l_pushresultsize(&mut b, total);

        1
    }
}
