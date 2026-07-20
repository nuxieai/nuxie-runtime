#[allow(non_snake_case)]
/// C++ `gnext(n)` = `(n)->key.next`. The bitfield-packed `next` has no stable
/// address, so this returns the value (the pointer form was a mis-port; unused).
pub unsafe fn gnext_n_mut(n: *mut crate::records::lua_node::LuaNode) -> i32 {
    (*n).key.next()
}
