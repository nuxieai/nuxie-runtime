use crate::functions::lua_l_typeerror_l::lua_l_typeerror_l;
use crate::functions::lua_typename::lua_typename;
use crate::type_aliases::lua_state::lua_State;

pub fn tag_error(L: *mut lua_State, narg: core::ffi::c_int, tag: core::ffi::c_int) -> ! {
    unsafe {
        let tname = lua_typename(L, tag);
        let tname = core::ffi::CStr::from_ptr(tname).to_string_lossy();
        lua_l_typeerror_l(L, narg, tname.as_ref())
    }
}
