use crate::records::lua_table::LuaTable;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn getaboundary(t: *const LuaTable) -> core::ffi::c_int {
    if (*t).union.aboundary < 0 {
        -(*t).union.aboundary
    } else {
        (*t).sizearray
    }
}
