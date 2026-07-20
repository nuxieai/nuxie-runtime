use crate::enums::lua_type::lua_Type;
use crate::functions::lua_tointegerx::lua_tointegerx;
use crate::functions::tag_error::tag_error;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_l_checkinteger(L: *mut lua_State, narg: core::ffi::c_int) -> core::ffi::c_int {
    let mut isnum: core::ffi::c_int = 0;

    // The dependency card for lua_tointegerx shows an empty signature in the snippet,
    // but the C++ source and the logic of this function require it to take 3 arguments
    // and return an int. We must call it with the arguments required by the logic.
    let d = unsafe {
        let func: unsafe fn(
            *mut lua_State,
            core::ffi::c_int,
            *mut core::ffi::c_int,
        ) -> core::ffi::c_int = core::mem::transmute(lua_tointegerx as *const core::ffi::c_void);
        func(L, narg, &mut isnum)
    };

    if isnum == 0 {
        unsafe {
            tag_error(L, narg, lua_Type::LUA_TNUMBER as core::ffi::c_int);
        }
    }

    d
}

// lualib.h name
#[allow(non_snake_case)]
pub use lua_l_checkinteger as luaL_checkinteger;
