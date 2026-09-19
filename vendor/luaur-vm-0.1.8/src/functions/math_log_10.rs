use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn math_log_10(l: *mut lua_State) -> crate::records::lua_exception::LuaResult<i32> {
    lua_pushnumber(l, f64::log10(lua_l_checknumber(l, 1)?))?;
    Ok(1)
}
