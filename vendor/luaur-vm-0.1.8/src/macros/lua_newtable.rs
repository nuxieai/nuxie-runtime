use crate::functions::lua_createtable::lua_createtable;

#[inline]
pub fn lua_newtable(
    l: *mut crate::records::lua_state::lua_State,
) -> crate::records::lua_exception::LuaResult<()> {
    unsafe { lua_createtable(l, 0, 0) }
}
