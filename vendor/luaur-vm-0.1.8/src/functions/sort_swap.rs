use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::setobj_2_t::setobj2t;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[inline]
pub unsafe fn sort_swap(L: *mut lua_State, t: *mut LuaTable, i: i32, j: i32) {
    let arr = (*t).array;
    let n = (*t).sizearray;

    LUAU_ASSERT!((i as u32) < (n as u32) && (j as u32) < (n as u32));

    let mut temp: TValue = core::mem::zeroed();
    setobj_2_s!(L, &mut temp as *mut TValue, arr.add(i as usize));
    setobj2t!(L, arr.add(i as usize), arr.add(j as usize));
    setobj2t!(L, arr.add(j as usize), &temp as *const TValue);
}
