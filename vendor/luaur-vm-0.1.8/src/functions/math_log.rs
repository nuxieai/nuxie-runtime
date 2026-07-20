use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::macros::lua_isnoneornil::lua_isnoneornil;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn math_log(l: *mut lua_State) -> i32 {
    let x = lua_l_checknumber(l, 1);
    let res = if lua_isnoneornil!(l, 2) {
        x.ln()
    } else {
        let base = lua_l_checknumber(l, 2);
        if base == 2.0 {
            x.log2()
        } else if base == 10.0 {
            x.log10()
        } else {
            x.ln() / base.ln()
        }
    };

    lua_pushnumber(l, res);
    1
}
