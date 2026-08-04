use crate::macros::api_incr_top::api_incr_top;
use crate::macros::setvvalue::setvvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[export_name = "luaur_lua_pushvector_lua_state_f32_f32_f32_f32"]
pub unsafe fn lua_pushvector_lua_state_f32_f32_f32_f32(
    l: *mut lua_State,
    x: crate::type_aliases::lua_vector_type::LuaVectorType,
    y: crate::type_aliases::lua_vector_type::LuaVectorType,
    z: crate::type_aliases::lua_vector_type::LuaVectorType,
    w: crate::type_aliases::lua_vector_type::LuaVectorType,
) {
    #[cfg(feature = "lua_vector_double")]
    {
        crate::macros::lua_c_check_gc::luaC_checkGC!(l);
        crate::macros::lua_c_threadbarrier::luaC_threadbarrier!(l);
    }
    crate::ensure_stack!(l, 1);
    setvvalue!(l, (*l).top, x, y, z, w);
    api_incr_top!(l);
}
