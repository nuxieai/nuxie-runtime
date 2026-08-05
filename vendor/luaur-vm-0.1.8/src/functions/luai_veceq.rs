use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_vector_type::LuaVectorType;

#[inline]
pub unsafe fn luai_veceq(a: *const LuaVectorType, b: *const LuaVectorType) -> bool {
    if LUA_VECTOR_SIZE == 4 {
        *a == *b && *a.add(1) == *b.add(1) && *a.add(2) == *b.add(2) && *a.add(3) == *b.add(3)
    } else {
        *a == *b && *a.add(1) == *b.add(1) && *a.add(2) == *b.add(2)
    }
}
