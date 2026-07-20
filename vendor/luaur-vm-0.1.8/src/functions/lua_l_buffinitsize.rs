use crate::functions::lua_l_buffinit::lua_l_buffinit;
use crate::functions::lua_l_prepbuffsize::lua_l_prepbuffsize;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

pub unsafe fn lua_l_buffinitsize(
    L: *mut lua_State,
    B: *mut LuaLStrbuf,
    size: usize,
) -> *mut c_char {
    lua_l_buffinit(L, B);
    lua_l_prepbuffsize(B, size)
}
