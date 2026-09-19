use crate::enums::lua_type::lua_Type;
use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::functions::lua_h_clear::lua_h_clear;
use crate::functions::lua_l_checktype::lua_l_checktype;
use crate::macros::hvalue::hvalue;
use crate::type_aliases::lua_state::lua_State;

#[export_name = "luaur_tclear"]
pub unsafe fn tclear(
    L: *mut lua_State,
) -> crate::records::lua_exception::LuaResult<core::ffi::c_int> {
    lua_l_checktype(L, 1, lua_Type::LUA_TTABLE as core::ffi::c_int)?;

    let tt = hvalue!((*L).base);

    if (*tt).readonly != 0 {
        return lua_g_readonlyerror(L);
    }

    lua_h_clear(tt);
    Ok(0)
}
