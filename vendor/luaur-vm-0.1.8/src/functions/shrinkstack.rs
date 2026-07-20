use crate::functions::lua_d_realloc_ci::lua_d_realloc_ci;
use crate::functions::lua_d_reallocstack::lua_d_reallocstack;
use crate::macros::basic_ci_size::BASIC_CI_SIZE;
use crate::macros::basic_stack_size::BASIC_STACK_SIZE;
use crate::macros::cast_int::cast_int;
use crate::macros::condhardstacktests::condhardstacktests;
use crate::macros::extra_stack::EXTRA_STACK;
use crate::macros::luai_maxcalls::LUAI_MAXCALLS;
use crate::records::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn shrinkstack(L: *mut lua_State) {
    // compute used stack - note that we can't use th->top if we're in the middle of vararg call
    let mut lim: StkId = (*L).top;
    let mut ci: *mut CallInfo = (*L).base_ci;
    while ci <= (*L).ci {
        LUAU_ASSERT!((*ci).top <= (*L).stack_last);
        if lim < (*ci).top {
            lim = (*ci).top;
        }
        ci = ci.add(1);
    }

    // shrink stack and callinfo arrays if we aren't using most of the space
    let ci_used = cast_int!((*L).ci.offset_from((*L).base_ci));
    let s_used = cast_int!(lim.offset_from((*L).stack));
    if (*L).size_ci > LUAI_MAXCALLS {
        // handling overflow?
        return;
    }

    if 3 * (ci_used as usize) < (*L).size_ci as usize && 2 * BASIC_CI_SIZE < (*L).size_ci {
        lua_d_realloc_ci(L, (*L).size_ci / 2); // still big enough...
    }

    condhardstacktests!(lua_d_realloc_ci(L, ci_used + 1));

    if 3 * (s_used as usize) < (*L).stacksize as usize
        && 2 * (BASIC_STACK_SIZE + EXTRA_STACK) < (*L).stacksize
    {
        lua_d_reallocstack(L, (*L).stacksize / 2, 0); // still big enough...
    }

    condhardstacktests!(lua_d_reallocstack(L, s_used, 0));
}
