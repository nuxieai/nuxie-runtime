use crate::functions::sort_heap::sort_heap;
use crate::functions::sort_less::sort_less;
use crate::functions::sort_swap::sort_swap;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::sort_predicate::SortPredicate;

pub fn sort_rec(
    L: *mut lua_State,
    t: *mut LuaTable,
    mut l: i32,
    mut u: i32,
    mut limit: i32,
    pred: SortPredicate,
) {
    // sort range [l..u] (inclusive, 0-based)
    while l < u {
        // if the limit has been reached, quick sort is going over the permitted nlogn complexity,
        // so we fall back to heap sort
        if limit == 0 {
            sort_heap(L, t, l, u, pred);
            return;
        }

        // sort elements a[l], a[(l+u)/2] and a[u]
        // note: this simultaneously acts as a small sort and a median selector
        unsafe {
            if sort_less(L, t, u, l, pred) != 0 {
                sort_swap(L, t, u, l);
            }
        }

        if u - l == 1 {
            break; // only 2 elements
        }

        let m = l + ((u - l) >> 1); // midpoint

        unsafe {
            if sort_less(L, t, m, l, pred) != 0 {
                sort_swap(L, t, m, l);
            } else if sort_less(L, t, u, m, pred) != 0 {
                sort_swap(L, t, m, u);
            }
        }

        if u - l == 2 {
            break; // only 3 elements
        }

        // here l, m, u are ordered; m will become the new pivot
        let p = u - 1;
        unsafe {
            sort_swap(L, t, m, u - 1); // pivot is now (and always) at u-1
        }

        // a[l] <= P == a[u-1] <= a[u], only need to sort from l+1 to u-2
        let mut i = l;
        let mut j = u - 1;

        loop {
            // invariant: a[l..i] <= P <= a[j..u]
            // repeat ++i until a[i] >= P
            loop {
                i += 1;
                unsafe {
                    if sort_less(L, t, i, p, pred) != 0 {
                        if i >= u {
                            crate::functions::lua_l_error_l::lua_l_error_l(
                                L,
                                c"invalid order function for sorting".as_ptr(),
                                core::format_args!("invalid order function for sorting"),
                            );
                        }
                        continue;
                    }
                }
                break;
            }

            // repeat --j until a[j] <= P
            loop {
                j -= 1;
                unsafe {
                    if sort_less(L, t, p, j, pred) != 0 {
                        if j <= l {
                            crate::functions::lua_l_error_l::lua_l_error_l(
                                L,
                                c"invalid order function for sorting".as_ptr(),
                                core::format_args!("invalid order function for sorting"),
                            );
                        }
                        continue;
                    }
                }
                break;
            }

            if j < i {
                break;
            }

            unsafe {
                sort_swap(L, t, i, j);
            }
        }

        // swap pivot a[p] with a[i], which is the new midpoint
        unsafe {
            sort_swap(L, t, p, i);
        }

        // adjust limit to allow 1.5 log2N recursive steps
        limit = (limit >> 1) + (limit >> 2);

        // a[l..i-1] <= a[i] == P <= a[i+1..u]
        // sort smaller half recursively; the larger half is sorted in the next loop iteration
        if i - l < u - i {
            sort_rec(L, t, l, i - 1, limit, pred);
            l = i + 1;
        } else {
            sort_rec(L, t, i + 1, u, limit, pred);
            u = i - 1;
        }
    }
}
