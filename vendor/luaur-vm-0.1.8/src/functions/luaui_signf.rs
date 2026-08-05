#[inline]
pub fn luaui_signf(v: crate::type_aliases::lua_vector_type::LuaVectorType) -> crate::type_aliases::lua_vector_type::LuaVectorType {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}
