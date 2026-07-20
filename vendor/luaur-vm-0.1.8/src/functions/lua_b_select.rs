use crate::enums::lua_type::lua_Type;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_pushinteger::lua_pushinteger;
use crate::functions::lua_type::lua_type;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_tostring::lua_tostring;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_b_select(L: *mut lua_State) -> core::ffi::c_int {
    let n = lua_gettop(L);
    let first_type = lua_type(L, 1);
    if first_type == lua_Type::LUA_TSTRING as i32 {
        let str_ptr = lua_tostring!(L, 1);
        let first_char = *str_ptr;
        if first_char == b'#' as core::ffi::c_char {
            lua_pushinteger(L, n - 1);
            return 1;
        }
    }

    let i = lua_l_checkinteger(L, 1);
    let i = if i < 0 {
        n + i
    } else if i > n {
        n
    } else {
        i
    };

    luaL_argcheck!(L, 1 <= i, 1, "index out of range");
    n - i
}
