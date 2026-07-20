//! Node: `cxx:Macro:Luau.VM:VM/src/lstate.h:incr_ci` (hand-checked)
//! C++ `incr_ci(L)` yields the NEW CallInfo: `luaD_growCI` advances `L->ci`
//! internally; the fast path advances it inline. Both branches end with the
//! macro evaluating to `(*L).ci` (the C++ ternary returned it directly; the
//! original Rust translation returned incompatible branch types and could
//! never have expanded).

#[allow(non_snake_case)]
#[macro_export]
macro_rules! incr_ci {
    ($L:expr) => {{
        let L = $L;
        if (*L).ci == (*L).end_ci {
            crate::functions::lua_d_grow_ci::luaD_growCI(L);
        } else {
            crate::macros::condhardstacktests::condhardstacktests!(
                crate::functions::lua_d_realloc_ci::luaD_reallocCI(L, (*L).size_ci)
            );
            (*L).ci = (*L).ci.wrapping_add(1);
        }
        (*L).ci
    }};
}

pub use incr_ci;
