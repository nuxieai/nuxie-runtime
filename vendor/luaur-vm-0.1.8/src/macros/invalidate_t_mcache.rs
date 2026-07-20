use crate::records::lua_table::LuaTable;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn invalidateTMcache(t: *mut LuaTable) {
    (*t).tmcache = 0;
}

#[allow(non_upper_case_globals)]
pub const invalidate_t_mcache: unsafe fn(*mut LuaTable) = invalidateTMcache;
