use crate::enums::lua_type::lua_Type;
use crate::functions::getfunc::getfunc;
use crate::functions::lua_insert::lua_insert;
use crate::functions::lua_iscfunction::lua_iscfunction;
use crate::functions::lua_isnumber::lua_isnumber;
use crate::functions::lua_l_checktype::lua_l_checktype;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::functions::lua_pushthread::lua_pushthread;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_setfenv::lua_setfenv;
use crate::functions::lua_setsafeenv::lua_setsafeenv;
use crate::macros::lua_tonumber::lua_tonumber;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_b_setfenv(L: *mut lua_State) -> i32 {
    lua_l_checktype(L, 2, lua_Type::LUA_TTABLE as i32);
    getfunc(L, 0);
    lua_pushvalue(L, 2);
    lua_setsafeenv(L, -1, 0);
    if lua_isnumber(L, 1) != 0 && lua_tonumber!(L, 1) == 0.0 {
        lua_pushthread(L);
        lua_insert(L, -2);
        lua_setfenv(L, -2);
        return 0;
    } else if lua_iscfunction(L, -2) != 0 || lua_setfenv(L, -2) == 0 {
        lua_l_error_l(
            L,
            c"'setfenv' cannot change environment of given object".as_ptr(),
            format_args!("'setfenv' cannot change environment of given object"),
        );
    }
    1
}
