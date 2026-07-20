//! Node: `cxx:Function:Luau.VM:VM/src/ldo.cpp:729:lua_d_pcall`
//! Source: `VM/src/ldo.cpp` (ldo.cpp:729-795, hand-ported)

use crate::enums::lua_status::lua_Status;
use crate::functions::callerrfunc::callerrfunc;
use crate::functions::lua_d_rawrunprotected_ldo_alt_b::lua_d_rawrunprotected_mut;
use crate::functions::lua_d_seterrorobj::luaD_seterrorobj;
use crate::functions::lua_f_close::lua_f_close as luaF_close;
use crate::functions::restore_stack_limit::restore_stack_limit;
use crate::macros::clvalue::clvalue;
use crate::macros::restoreci::restoreci;
use crate::macros::restorestack::restorestack;
use crate::macros::saveci::saveci;
use crate::records::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::pfunc::Pfunc;
use crate::type_aliases::stk_id::StkId;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaD_pcall(
    L: *mut lua_State,
    func: Pfunc,
    u: *mut core::ffi::c_void,
    old_top: isize,
    ef: isize,
) -> i32 {
    let oldnCcalls: u16 = (*L).nCcalls;
    let oldbaseCcalls: u16 = (*L).baseCcalls;
    let old_ci: isize = saveci!(L, (*L).ci);
    let oldactive: bool = (*L).isactive;
    let mut status: i32 = lua_d_rawrunprotected_mut(L, func, u);
    if status != 0 {
        let mut errstatus: i32 = status;

        if luaur_common::FFlag::LuauClosureUsageCounter.get() {
            let mut lastci: *mut CallInfo = (*L).ci;
            let savedci: *mut CallInfo = restoreci!(L, old_ci);
            while lastci != savedci {
                let cl =
                    clvalue!((*lastci).func) as *const _ as *mut crate::records::closure::Closure;
                LUAU_ASSERT!((*cl).usage > 0);
                (*cl).usage -= 1;
                lastci = lastci.offset(-1);
            }
        }

        // call user-defined error function (used in xpcall)
        if ef != 0 {
            // push error object to stack top if it's not already there
            if status != lua_Status::LUA_ERRRUN as i32 {
                luaD_seterrorobj(L, status, (*L).top);
            }

            // if errfunc fails, we fail with "error in error handling" or "not enough memory"
            let err = lua_d_rawrunprotected_mut(
                L,
                Some(callerrfunc),
                restorestack!(L, ef) as *mut core::ffi::c_void,
            );

            // in general we preserve the status, except for cases when the error handler fails
            // out of memory is treated specially because it's common for it to be cascading, in which case we preserve the code
            if err == 0 {
                errstatus = lua_Status::LUA_ERRRUN as i32;
            } else if status == lua_Status::LUA_ERRMEM as i32
                && err == lua_Status::LUA_ERRMEM as i32
            {
                errstatus = lua_Status::LUA_ERRMEM as i32;
            } else {
                errstatus = lua_Status::LUA_ERRERR as i32;
                status = lua_Status::LUA_ERRERR as i32;
                LUAU_ASSERT!(errstatus != 0);
            }
        }

        // since the call failed with an error, we might have to reset the 'active' thread state
        if !oldactive {
            (*L).isactive = false;
        }

        // Inlined logic from 'lua_isyieldable' to avoid potential for an out of line call.
        let yieldable: bool = (*L).nCcalls <= (*L).baseCcalls;

        // restore nCcalls and baseCcalls before calling the debugprotectederror callback which may rely on the proper value to have been restored.
        (*L).nCcalls = oldnCcalls;
        (*L).baseCcalls = oldbaseCcalls;

        // an error occurred, check if we have a protected error callback
        if yieldable {
            if let Some(debugprotectederror) = (*(*L).global).cb.debugprotectederror {
                debugprotectederror(L);

                // debug hook is only allowed to break
                if (*L).status as i32 == lua_Status::LUA_BREAK as i32 {
                    return 0;
                }
            }
        }

        let oldtop: StkId = restorestack!(L, old_top);
        luaF_close(L, oldtop); // close eventual pending closures
        luaD_seterrorobj(L, errstatus, oldtop);
        (*L).ci = restoreci!(L, old_ci);
        (*L).base = (*(*L).ci).base;
        restore_stack_limit(L);
    }
    status
}

#[allow(unused_imports)]
pub use luaD_pcall as lua_d_pcall;
