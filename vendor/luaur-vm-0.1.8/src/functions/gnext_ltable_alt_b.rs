/// C++ `gnext(mp)` value accessor (bitfield `next` has no stable address).
pub unsafe fn gnext_mp(mp: *mut crate::records::lua_node::LuaNode) -> i32 {
    (*mp).key.next()
}
