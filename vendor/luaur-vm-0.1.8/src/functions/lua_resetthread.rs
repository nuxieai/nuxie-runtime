//! Node: `cxx:Function:Luau.VM:VM/src/lstate.cpp:148:lua_resetthread`
//! Source: `VM/src/lstate.cpp:148-180` (hand-ported)

use crate::enums::lua_status::lua_Status;
use crate::functions::cleanupcistack::cleanupcistack;
use crate::functions::lua_d_realloc_ci::lua_d_realloc_ci;
use crate::functions::lua_d_reallocstack::luaD_reallocstack;
use crate::functions::lua_f_close::lua_f_close;
use crate::macros::api_check::api_check;
use crate::macros::basic_ci_size::BASIC_CI_SIZE;
use crate::macros::basic_stack_size::BASIC_STACK_SIZE;
use crate::macros::extra_stack::EXTRA_STACK;
use crate::macros::lua_minstack::LUA_MINSTACK;
use crate::macros::setnilvalue::setnilvalue;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_resetthread(L: *mut lua_State) {
    api_check!(L, !(*L).isactive);
    api_check!(
        L,
        (*L).status != lua_Status::LUA_OK as u8 || (*L).ci == (*L).base_ci
    );

    // close upvalues before clearing anything
    lua_f_close(L, (*L).stack);
    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
        cleanupcistack(L);
    }

    // clear call frames
    let ci = (*L).base_ci;
    (*ci).func = (*L).stack;
    (*ci).base = (*ci).func.add(1);
    (*ci).top = (*ci).base.add(LUA_MINSTACK as usize);
    setnilvalue!((*ci).func);
    (*L).ci = ci;
    if (*L).size_ci != BASIC_CI_SIZE {
        lua_d_realloc_ci(L, BASIC_CI_SIZE);
    }
    // clear thread state
    (*L).status = lua_Status::LUA_OK as u8;
    (*L).base = (*(*L).ci).base;
    (*L).top = (*(*L).ci).base;
    (*L).nCcalls = 0;
    (*L).baseCcalls = 0;
    // clear thread stack
    if (*L).stacksize != BASIC_STACK_SIZE + EXTRA_STACK {
        luaD_reallocstack(L, BASIC_STACK_SIZE, 0);
    }
    for i in 0..(*L).stacksize as usize {
        setnilvalue!((*L).stack.add(i));
    }
}
