use core::ffi::{c_char, CStr};

use crate::functions::currfuncname::currfuncname;
use crate::functions::lua_a_toobject::luaA_toobject;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_l_typeerror_l(L: *mut lua_State, narg: core::ffi::c_int, tname: &str) -> ! {
    let fname: *const c_char = currfuncname(L);
    let obj: *const TValue = luaA_toobject(L, narg);

    if !obj.is_null() {
        let objtypename = lua_t_objtypename(L, obj);
        let objtypename = CStr::from_ptr(objtypename).to_string_lossy();

        if !fname.is_null() {
            let fname = CStr::from_ptr(fname).to_string_lossy();
            lua_l_error_l(
                L,
                c"invalid argument #%d to '%s' (%s expected, got %s)".as_ptr(),
                format_args!(
                    "invalid argument #{} to '{}' ({} expected, got {})",
                    narg, fname, tname, objtypename
                ),
            );
        } else {
            lua_l_error_l(
                L,
                c"invalid argument #%d (%s expected, got %s)".as_ptr(),
                format_args!(
                    "invalid argument #{} ({} expected, got {})",
                    narg, tname, objtypename
                ),
            );
        }
    } else {
        if !fname.is_null() {
            let fname = CStr::from_ptr(fname).to_string_lossy();
            lua_l_error_l(
                L,
                c"missing argument #%d to '%s' (%s expected)".as_ptr(),
                format_args!(
                    "missing argument #{} to '{}' ({} expected)",
                    narg, fname, tname
                ),
            );
        } else {
            lua_l_error_l(
                L,
                c"missing argument #%d (%s expected)".as_ptr(),
                format_args!("missing argument #{} ({} expected)", narg, tname),
            );
        }
    }

    core::hint::unreachable_unchecked()
}
