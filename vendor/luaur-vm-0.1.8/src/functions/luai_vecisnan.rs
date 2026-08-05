#[inline]
pub fn luai_vecisnan(a: *const crate::type_aliases::lua_vector_type::LuaVectorType) -> bool {
    if crate::macros::lua_vector_size::LUA_VECTOR_SIZE == 4 {
        unsafe {
            let v0 = *a;
            let v1 = *a.add(1);
            let v2 = *a.add(2);
            let v3 = *a.add(3);
            v0 != v0 || v1 != v1 || v2 != v2 || v3 != v3
        }
    } else {
        unsafe {
            let v0 = *a;
            let v1 = *a.add(1);
            let v2 = *a.add(2);
            v0 != v0 || v1 != v1 || v2 != v2
        }
    }
}
