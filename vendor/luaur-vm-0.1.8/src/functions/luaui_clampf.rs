#[inline]
pub fn luaui_clampf(v: crate::type_aliases::lua_vector_type::LuaVectorType, min: crate::type_aliases::lua_vector_type::LuaVectorType, max: crate::type_aliases::lua_vector_type::LuaVectorType) -> crate::type_aliases::lua_vector_type::LuaVectorType {
    let r = if v < min { min } else { v };
    if r > max {
        max
    } else {
        r
    }
}
