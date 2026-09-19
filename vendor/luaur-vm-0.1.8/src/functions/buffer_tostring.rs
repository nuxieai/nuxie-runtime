use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::type_aliases::lua_state::lua_State;

pub fn buffer_tostring(
    L: *mut lua_State,
) -> crate::records::lua_exception::LuaResult<core::ffi::c_int> {
    let mut len: usize = 0;
    let data = lua_l_checkbuffer(L, 1, &mut len)?;

    unsafe {
        lua_pushlstring(L, data as *const core::ffi::c_char, len)?;
    }

    Ok(1)
}
