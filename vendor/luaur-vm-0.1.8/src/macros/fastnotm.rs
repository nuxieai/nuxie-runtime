use crate::records::lua_table::LuaTable;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn fastnotm(et: *mut LuaTable, e: i32) -> bool {
    et.is_null() || ((*et).tmcache as i32 & (1 << e)) != 0
}

#[allow(non_upper_case_globals)]
pub const fast_no_tm: unsafe fn(*mut LuaTable, i32) -> bool = fastnotm;
