use crate::functions::currfuncname::currfuncname;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub unsafe fn lua_l_argerror_l<T>(
    L: *mut lua_State,
    narg: c_int,
    extramsg: &str,
) -> crate::records::lua_exception::LuaResult<T> {
    let fname = currfuncname(L);

    if !fname.is_null() {
        let fname = core::ffi::CStr::from_ptr(fname).to_string_lossy();
        lua_l_error_l(
            L,
            c"invalid argument #%d to '%s' (%s)".as_ptr(),
            format_args!("invalid argument #{} to '{}' ({})", narg, fname, extramsg),
        )
    } else {
        lua_l_error_l(
            L,
            c"invalid argument #%d (%s)".as_ptr(),
            format_args!("invalid argument #{} ({})", narg, extramsg),
        )
    }
}

#[allow(non_snake_case)]
pub fn luaL_argerrorL<T>(
    L: *mut lua_State,
    narg: c_int,
    extramsg: &str,
) -> crate::records::lua_exception::LuaResult<T> {
    unsafe { lua_l_argerror_l(L, narg, extramsg) }
}
