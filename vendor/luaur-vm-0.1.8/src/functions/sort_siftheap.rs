use crate::functions::sort_less::sort_less;
use crate::functions::sort_swap::sort_swap;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::sort_predicate::SortPredicate;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn sort_siftheap(
    L: *mut lua_State,
    t: *mut LuaTable,
    l: i32,
    u: i32,
    pred: SortPredicate,
    root: i32,
) -> crate::records::lua_exception::LuaResult<()> {
    LUAU_ASSERT!(l <= u);
    let count = u - l + 1;

    // process all elements with two children
    let mut root = root;
    while root * 2 + 2 < count {
        let left = root * 2 + 1;
        let right = root * 2 + 2;
        let mut next = root;

        next = if unsafe { sort_less(L, t, l + next, l + left, pred)? } != 0 {
            left
        } else {
            next
        };
        next = if unsafe { sort_less(L, t, l + next, l + right, pred)? } != 0 {
            right
        } else {
            next
        };

        if next == root {
            break;
        }

        unsafe {
            sort_swap(L, t, l + root, l + next);
        }
        root = next;
    }

    // process last element if it has just one child
    let lastleft = root * 2 + 1;
    if lastleft == count - 1 {
        if unsafe { sort_less(L, t, l + root, l + lastleft, pred)? } != 0 {
            unsafe {
                sort_swap(L, t, l + root, l + lastleft);
            }
        }
    }
    Ok(())
}
