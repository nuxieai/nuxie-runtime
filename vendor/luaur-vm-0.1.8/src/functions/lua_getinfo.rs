//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:185:lua_getinfo`
//!
//! `lua_getinfo` — resolve a stack `level` (negative = relative to top, else a
//! call-info depth) to its closure, fill `ar` via `auxgetinfo`, and (when the
//! `f` option pushed the function) place it on the stack. Returns 1 if a
//! function was found at that level, else 0.

use crate::functions::auxgetinfo::auxgetinfo;
use crate::macros::clvalue::clvalue;
use crate::macros::incr_top::incr_top;
use crate::macros::lua_c_threadbarrier::luaC_threadbarrier;
use crate::macros::setclvalue::setclvalue;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::call_info::CallInfo;
use crate::records::closure::Closure;
use crate::records::lua_debug::LuaDebug;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};
use luaur_common::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_getinfo(
    L: *mut lua_State,
    level: c_int,
    what: *const c_char,
    ar: *mut LuaDebug,
) -> c_int {
    let mut f: *mut Closure = core::ptr::null_mut();
    let mut ci: *mut CallInfo = core::ptr::null_mut();

    if level < 0 {
        // element has to be within stack
        if (-level) as isize > (*L).top.offset_from((*L).base) {
            return 0;
        }

        let func = (*L).top.offset(level as isize);

        // and it has to be a function
        if !ttisfunction!(func) {
            return 0;
        }

        f = clvalue!(func);
    } else if (level as u32) < (*L).ci.offset_from((*L).base_ci) as u32 {
        ci = (*L).ci.offset(-(level as isize));
        LUAU_ASSERT!(ttisfunction!((*ci).func));
        f = clvalue!((*ci).func);
    }

    if !f.is_null() {
        // auxgetinfo fills ar and optionally requests to put closure on stack
        let fcl = auxgetinfo(L, what, ar, f, ci);
        if !fcl.is_null() {
            luaC_threadbarrier!(L);
            setclvalue!(L, (*L).top, fcl);
            incr_top!(L);
        }
    }

    if f.is_null() {
        0
    } else {
        1
    }
}
