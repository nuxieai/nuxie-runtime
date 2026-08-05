#[inline]
pub fn luai_lerpf(a: crate::type_aliases::lua_vector_type::LuaVectorType, b: crate::type_aliases::lua_vector_type::LuaVectorType, t: crate::type_aliases::lua_vector_type::LuaVectorType) -> crate::type_aliases::lua_vector_type::LuaVectorType {
    if t == 1.0 {
        b
    } else {
        a + (b - a) * t
    }
}
