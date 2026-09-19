use crate::functions::lua_insert::lua_insert;
use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::type_aliases::lua_state::lua_State;

#[export_name = "luaur_coresumefinish"]
pub unsafe fn coresumefinish(
    L: *mut lua_State,
    r: core::ffi::c_int,
) -> crate::records::lua_exception::LuaResult<core::ffi::c_int> {
    if r < 0 {
        lua_pushboolean(L, 0)?;
        lua_insert(L, -2);
        Ok(2)
    } else {
        lua_pushboolean(L, 1)?;
        lua_insert(L, -(r + 1));
        Ok(r + 1)
    }
}
