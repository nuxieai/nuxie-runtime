use crate::functions::lua_d_check_cstack::luaD_checkCstack;
use crate::functions::performcall::performcall;
use crate::macros::clvalue::clvalue;
use crate::macros::isyielded::isyielded;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::macros::luai_maxccalls::LUAI_MAXCCALLS;
use crate::macros::restoreci::restoreci;
use crate::macros::restorestack::restorestack;
use crate::macros::saveci::saveci;
use crate::macros::savestack::savestack;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn lua_d_callint(l: *mut lua_State, func: StkId, nresults: c_int, preparereentry: bool) {
    (*l).nCcalls = (*l).nCcalls.wrapping_add(1);
    if (*l).nCcalls as i32 >= LUAI_MAXCCALLS {
        luaD_checkCstack(l);
    }

    let mut fromyieldableccall = false;

    if (*l).ci != (*l).base_ci {
        let ccl = clvalue!((*(*l).ci).func);
        let cc = core::ptr::addr_of!((*ccl).inner.c).cast::<crate::records::closure::CClosure>();
        if (*ccl).isC != 0 && (*cc).cont.is_some() {
            fromyieldableccall = true;
            (*l).baseCcalls = (*l).baseCcalls.wrapping_add(1);
        }
    }

    let funcoffset = savestack!(l, func);
    let cioffset = saveci!(l, (*l).ci);

    performcall(l, func, nresults, preparereentry);

    let yielded = isyielded(l);

    if fromyieldableccall {
        (*l).baseCcalls = (*l).baseCcalls.wrapping_sub(1);

        if yielded {
            let callerci = restoreci!(l, cioffset);
            (*callerci).top = restorestack!(l, funcoffset).add(if nresults != LUA_MULTRET {
                nresults as usize
            } else {
                0
            });
        }
    }

    if nresults != LUA_MULTRET && !yielded {
        (*l).top = restorestack!(l, funcoffset).add(nresults as usize);
    }

    (*l).nCcalls = (*l).nCcalls.wrapping_sub(1);
    luaC_checkGC!(l);
}

#[allow(unused_imports)]
pub use lua_d_callint as luaD_callint;
