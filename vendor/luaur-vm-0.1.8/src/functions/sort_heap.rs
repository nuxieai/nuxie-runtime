use crate::functions::sort_siftheap::sort_siftheap;
use crate::functions::sort_swap::sort_swap;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::sort_predicate::SortPredicate;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn sort_heap(L: *mut lua_State, t: *mut LuaTable, l: i32, u: i32, pred: SortPredicate) {
    LUAU_ASSERT!(l <= u);
    let count = u - l + 1;

    let mut i = count / 2 - 1;
    while i >= 0 {
        sort_siftheap(L, t, l, u, pred, i);
        i -= 1;
    }

    let mut i = count - 1;
    while i > 0 {
        unsafe {
            sort_swap(L, t, l, l + i);
        }
        sort_siftheap(L, t, l, l + i - 1, pred, 0);
        i -= 1;
    }
}
