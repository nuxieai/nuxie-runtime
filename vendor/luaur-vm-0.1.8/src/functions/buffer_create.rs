use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_newbuffer::lua_newbuffer;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::type_aliases::lua_state::lua_State;

pub fn buffer_create(
    L: *mut lua_State,
) -> crate::records::lua_exception::LuaResult<core::ffi::c_int> {
    unsafe {
        let size = lua_l_checkinteger(L, 1)?;

        luaL_argcheck!(L, size >= 0, 1, "size");

        lua_newbuffer(L, size as usize)?;
        Ok(1)
    }
}
