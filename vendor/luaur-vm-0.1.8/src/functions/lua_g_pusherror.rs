use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::functions::pusherror::pusherror;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

#[export_name = "luaur_lua_g_pusherror"]
pub unsafe fn lua_g_pusherror(l: *mut lua_State, error: *const c_char) -> crate::records::lua_exception::LuaResult<()> {
    lua_rawcheckstack(l, 1)?;
    pusherror(l, if error.is_null() { c"".as_ptr() } else { error })
}

#[allow(non_snake_case)]
pub use lua_g_pusherror as luaG_pusherror;
