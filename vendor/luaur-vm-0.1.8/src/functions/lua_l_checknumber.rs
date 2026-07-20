use crate::enums::lua_type::lua_Type;
use crate::functions::lua_tonumberx::lua_tonumberx;
use crate::functions::tag_error::tag_error;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_l_checknumber(L: *mut lua_State, narg: core::ffi::c_int) -> f64 {
    let mut isnum: core::ffi::c_int = 0;
    let d = unsafe { lua_tonumberx(L, narg, &mut isnum) };
    if isnum == 0 {
        unsafe {
            tag_error(L, narg, lua_Type::LUA_TNUMBER as core::ffi::c_int);
        }
    }
    d
}

// lualib.h name
#[allow(non_snake_case)]
pub use lua_l_checknumber as luaL_checknumber;
