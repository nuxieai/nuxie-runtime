use crate::macros::gfasttm::gfasttm;
use crate::records::lua_state::lua_State;
use crate::records::lua_t_value::TValue;
use crate::records::lua_table::LuaTable;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn fasttm(l: *mut lua_State, et: *mut LuaTable, e: i32) -> *const TValue {
    gfasttm((*l).global, et, e)
}

#[allow(non_upper_case_globals)]
pub const fasttm_macro: unsafe fn(*mut lua_State, *mut LuaTable, i32) -> *const TValue = fasttm;
