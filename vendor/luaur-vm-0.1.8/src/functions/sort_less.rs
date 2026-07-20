use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::sort_predicate::SortPredicate;
use crate::type_aliases::t_value::TValue;

use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[inline]
pub unsafe fn sort_less(
    L: *mut lua_State,
    t: *mut LuaTable,
    i: i32,
    j: i32,
    pred: SortPredicate,
) -> i32 {
    let arr = (*t).array;
    let n = (*t).sizearray;

    LUAU_ASSERT!((i as u32) < (n as u32) && (j as u32) < (n as u32));

    let res = match pred {
        Some(f) => f(L, arr.add(i as usize), arr.add(j as usize)),
        None => 0,
    };

    // predicate call may resize the table, which is invalid
    if (*t).sizearray != n {
        lua_l_error_l(
            L,
            c"table modified during sorting".as_ptr(),
            core::format_args!("table modified during sorting"),
        );
    }

    res
}
