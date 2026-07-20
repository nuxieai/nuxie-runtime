#[allow(non_snake_case)]
pub unsafe fn gnext_n(n: *mut crate::records::lua_node::LuaNode) {
    (*n).key.set_next(0);
}
