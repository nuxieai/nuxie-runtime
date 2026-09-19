use crate::functions::lua_pushlightuserdatatagged::lua_pushlightuserdatatagged;

#[allow(non_upper_case_globals)]
pub const lua_pushlightuserdata: unsafe fn(
    *mut crate::records::lua_state::lua_State,
    *mut core::ffi::c_void,
) -> crate::records::lua_exception::LuaResult<()> = |l, p| lua_pushlightuserdatatagged(l, p, 0);
