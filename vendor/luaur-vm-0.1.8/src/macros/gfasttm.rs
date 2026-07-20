use crate::functions::lua_t_gettm::lua_t_gettm;
use crate::records::global_state::global_State;
use crate::records::lua_t_value::TValue;
use crate::records::lua_table::LuaTable;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn gfasttm(g: *mut global_State, et: *mut LuaTable, e: i32) -> *const TValue {
    if et.is_null() {
        core::ptr::null()
    } else {
        let tmcache = (*et).tmcache;
        // The C++ macro: ((et)->tmcache & (1u << (e))) ? NULL : luaT_gettm(...)
        // This means if the bit IS set, we return NULL (it's a "fast" check for absence).
        if (tmcache as u32 & (1u32 << e)) != 0 {
            core::ptr::null()
        } else {
            lua_t_gettm(et, core::mem::transmute(e), (*g).tmname[e as usize])
        }
    }
}

#[allow(non_upper_case_globals)]
pub const gfasttm_macro: unsafe fn(*mut global_State, *mut LuaTable, i32) -> *const TValue =
    gfasttm;
