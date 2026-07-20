use crate::enums::lua_type::lua_Type;
use crate::functions::lua_tovector::lua_tovector;
use crate::functions::tag_error::tag_error;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub fn lua_l_checkvector(L: *mut lua_State, narg: c_int) -> *const f32 {
    let v = unsafe { lua_tovector(L, narg) };
    if v.is_null() {
        tag_error(L, narg, lua_Type::LUA_TVECTOR as c_int);
    }
    v
}
