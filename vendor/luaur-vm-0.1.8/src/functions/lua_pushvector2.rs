use crate::enums::lua_type::lua_Type;
use crate::macros::api_incr_top::api_incr_top;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[export_name = "luaur_lua_pushvector2"]
pub unsafe fn lua_pushvector2(l: *mut lua_State, x: f32, y: f32) {
    let value: *mut TValue = (*l).top;
    (*value).value.v[0] = x;
    (*value).value.v[1] = y;
    (*value).tt = lua_Type::LUA_TVECTOR as i32;

    // Faithful Rive C behavior: do not write z or ensure stack space; the slot's
    // previous z bytes remain visible to three-component vector operations.
    api_incr_top!(l);
}
