//! Node: `cxx:Function:Luau.VM:VM/src/lvmexecute.cpp:206:luau_setupcci`
//!
//! Push and initialize a fresh `CallInfo` for a C continuation call: point it at
//! `fun`, give it a `LUA_MINSTACK` window, clear the saved pc/flags, bump the
//! closure usage counter (when enabled), and ensure the stack has room.

use crate::macros::clvalue::clvalue;
use crate::macros::incr_ci::incr_ci;
use crate::macros::lua_d_checkstackfornewci::luaD_checkstackfornewci;
use crate::macros::lua_minstack::LUA_MINSTACK;
use crate::macros::ttisfunction::ttisfunction;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use core::ffi::c_int;
use luaur_common::LUAU_ASSERT;

pub fn luau_setupcci(L: *mut lua_State, nresults: c_int, fun: StkId) {
    unsafe {
        let ci = incr_ci!(L);

        (*ci).func = fun;
        (*ci).base = fun.add(1);
        (*ci).top = (*L).top.add(LUA_MINSTACK as usize);
        (*ci).savedpc = core::ptr::null();
        (*ci).flags = 0;
        (*ci).nresults = nresults;

        if luaur_common::FFlag::LuauClosureUsageCounter.get() {
            (*clvalue!(fun)).usage += 1;
        }

        (*L).base = fun.add(1);

        luaD_checkstackfornewci(L, LUA_MINSTACK);

        LUAU_ASSERT!((*ci).top <= (*L).stack_last);
        LUAU_ASSERT!(ttisfunction!((*ci).func));
    }
}
