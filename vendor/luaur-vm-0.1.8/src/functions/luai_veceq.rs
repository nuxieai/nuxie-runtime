use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;

#[inline]
pub unsafe fn luai_veceq(a: *const f32, b: *const f32) -> bool {
    if LUA_VECTOR_SIZE == 4 {
        *a == *b && *a.add(1) == *b.add(1) && *a.add(2) == *b.add(2) && *a.add(3) == *b.add(3)
    } else {
        *a == *b && *a.add(1) == *b.add(1) && *a.add(2) == *b.add(2)
    }
}
