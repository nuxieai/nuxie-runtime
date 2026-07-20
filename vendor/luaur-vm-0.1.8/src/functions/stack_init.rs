//! Node: `cxx:Function:Luau.VM:VM/src/lstate.cpp:28:stack_init`
//! Source: `VM/src/lstate.cpp:28-48` (hand-ported)

use crate::macros::basic_ci_size::BASIC_CI_SIZE;
use crate::macros::basic_stack_size::BASIC_STACK_SIZE;
use crate::macros::extra_stack::EXTRA_STACK;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::macros::lua_minstack::LUA_MINSTACK;
use crate::macros::setnilvalue::setnilvalue;
use crate::records::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

pub unsafe fn stack_init(L1: *mut lua_State, L: *mut lua_State) {
    // initialize CallInfo array
    (*L1).base_ci = luaM_newarray!(L, BASIC_CI_SIZE, CallInfo, (*L1).hdr.memcat);
    (*L1).ci = (*L1).base_ci;
    (*L1).size_ci = BASIC_CI_SIZE;
    (*L1).end_ci = (*L1).base_ci.add((*L1).size_ci as usize - 1);
    // initialize stack array
    (*L1).stack = luaM_newarray!(L, BASIC_STACK_SIZE + EXTRA_STACK, TValue, (*L1).hdr.memcat);
    (*L1).stacksize = BASIC_STACK_SIZE + EXTRA_STACK;
    let stack = (*L1).stack;
    for i in 0..(BASIC_STACK_SIZE + EXTRA_STACK) as usize {
        setnilvalue!(stack.add(i)); // erase new stack
    }
    (*L1).top = stack;
    (*L1).stack_last = stack.add(((*L1).stacksize - EXTRA_STACK) as usize);
    // initialize first ci
    (*(*L1).ci).func = (*L1).top;
    setnilvalue!((*L1).top); // `function' entry for this `ci'
    (*L1).top = (*L1).top.add(1);
    (*L1).base = (*L1).top;
    (*(*L1).ci).base = (*L1).top;
    (*(*L1).ci).top = (*L1).top.add(LUA_MINSTACK as usize);
}
