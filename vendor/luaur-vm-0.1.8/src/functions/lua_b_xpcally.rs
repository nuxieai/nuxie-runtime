use crate::enums::lua_type::lua_Type;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checktype::lua_l_checktype;
use crate::functions::lua_l_pcallyieldable::lua_l_pcallyieldable;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_replace::lua_replace;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_b_xpcally(L: *mut lua_State) -> i32 {
    lua_l_checktype(L, 2, lua_Type::LUA_TFUNCTION as i32);

    // swap function & error function
    lua_pushvalue(L, 1);
    lua_pushvalue(L, 2);
    lua_replace(L, 1);
    lua_replace(L, 2);
    // at this point the stack looks like err, f, args

    lua_l_pcallyieldable(L, lua_gettop(L) - 2, LUA_MULTRET, 1)
}
