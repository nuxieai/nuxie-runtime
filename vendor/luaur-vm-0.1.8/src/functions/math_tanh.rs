use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::type_aliases::lua_state::lua_State;

#[export_name = "luaur_math_tanh"]
pub unsafe fn math_tanh(l: *mut lua_State) -> crate::records::lua_exception::LuaResult<i32> {
    lua_pushnumber(l, f64::tanh(lua_l_checknumber(l, 1)?))?;
    Ok(1)
}
