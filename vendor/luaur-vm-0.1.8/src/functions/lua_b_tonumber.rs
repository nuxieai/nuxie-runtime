use crate::functions::lua_l_checkany::lua_l_checkany;
use crate::functions::lua_l_optinteger::lua_l_optinteger;
use crate::functions::lua_pushnil::lua_pushnil;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::functions::lua_tonumberx::lua_tonumberx;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_checkstring::luaL_checkstring;
use crate::type_aliases::lua_state::lua_State;

// Helper for isspace: check if a u8 value corresponds to an ASCII whitespace character
#[inline]
unsafe fn isspace(c: u8) -> bool {
    match c {
        b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r' => true,
        _ => false,
    }
}

// Helper function for strtoull-like behavior via libc-compatible symbol
unsafe fn strtoull(
    s: *const core::ffi::c_char,
    endptr: &mut *mut core::ffi::c_char,
    base: u32,
) -> u64 {
    extern "C" {
        fn strtoull(
            s: *const core::ffi::c_char,
            endptr: *mut *mut core::ffi::c_char,
            base: u32,
        ) -> u64;
    }
    strtoull(s, endptr as *mut *mut core::ffi::c_char, base)
}

pub unsafe fn lua_b_tonumber(L: *mut lua_State) -> i32 {
    let base = lua_l_optinteger(L, 2, 10);

    if base == 10 {
        // standard conversion
        let mut isnum: core::ffi::c_int = 0;
        let n = lua_tonumberx(L, 1, &mut isnum);
        if isnum != 0 {
            lua_pushnumber(L, n);
            return 1;
        }
        lua_l_checkany(L, 1); // error if we don't have any argument
    } else {
        let s1 = luaL_checkstring!(L, 1);
        luaL_argcheck!(L, 2 <= base && base <= 36, 2, "base out of range");

        let mut s2: *mut core::ffi::c_char = core::ptr::null_mut();
        let n = strtoull(s1, &mut s2, base as u32);

        if s1 != s2 {
            // at least one valid digit?
            while isspace(*s2 as u8) {
                s2 = s2.add(1);
            } // skip trailing spaces

            if *s2 == b'\0' as core::ffi::c_char {
                // no invalid trailing characters?
                lua_pushnumber(L, n as f64);
                return 1;
            }
        }
    }

    lua_pushnil(L); // else not a number
    1
}
