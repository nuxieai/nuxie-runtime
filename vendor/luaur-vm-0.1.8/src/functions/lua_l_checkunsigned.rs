use crate::enums::lua_type::lua_Type;
use crate::functions::lua_tounsignedx::lua_tounsignedx;
use crate::functions::tag_error::tag_error;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_l_checkunsigned(L: *mut lua_State, narg: core::ffi::c_int) -> core::ffi::c_uint {
    let mut isnum: core::ffi::c_int = 0;
    let d = unsafe { lua_tounsignedx(L, narg, &mut isnum) };
    if isnum == 0 {
        unsafe {
            tag_error(L, narg, lua_Type::LUA_TNUMBER as core::ffi::c_int);
        }
    }
    d
}
