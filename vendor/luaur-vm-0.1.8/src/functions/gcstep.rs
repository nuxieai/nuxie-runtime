use crate::functions::atomic::atomic;
use crate::functions::lua_clock::lua_clock;
use crate::functions::lua_m_getnextpage::luaM_getnextpage;
use crate::functions::markroot::markroot;
use crate::functions::propagatemark::propagatemark;
use crate::functions::shrinkbuffers::shrinkbuffers;
use crate::functions::sweepgcopage::sweepgcopage;
use crate::macros::gc_satomic::{GCSatomic, GCSsweep};
use crate::macros::gc_spause::GCSpause;
use crate::macros::gc_spropagate::{GCSpropagate, GCSpropagateagain};
use crate::macros::gc_sweeppagestepcost::GC_SWEEPPAGESTEPCOST;
use crate::macros::makewhite::makewhite;
use crate::records::gc_object::GCObject;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn gcstep(l: *mut lua_State, limit: usize) -> usize {
    let mut cost = 0usize;
    let g = (*l).global;

    match (*g).gcstate as i32 {
        GCSpause => {
            markroot(l);
            LUAU_ASSERT!((*g).gcstate as i32 == GCSpropagate);
        }
        GCSpropagate => {
            while !(*g).gray.is_null() && cost < limit {
                cost += propagatemark(g);
            }

            if (*g).gray.is_null() {
                (*g).gray = (*g).grayagain;
                (*g).grayagain = core::ptr::null_mut();
                (*g).gcstate = GCSpropagateagain as u8;
            }
        }
        GCSpropagateagain => {
            while !(*g).gray.is_null() && cost < limit {
                cost += propagatemark(g);
            }

            if (*g).gray.is_null() {
                (*g).gcstate = GCSatomic as u8;
            }
        }
        GCSatomic => {
            (*g).gcstats.atomicstarttimestamp = lua_clock();
            (*g).gcstats.atomicstarttotalsizebytes = (*g).totalbytes;

            cost = atomic(l);

            LUAU_ASSERT!((*g).gcstate as i32 == GCSsweep);
        }
        GCSsweep => {
            while !(*g).sweepgcopage.is_null() && cost < limit {
                let next = luaM_getnextpage((*g).sweepgcopage);
                let steps = sweepgcopage(l, (*g).sweepgcopage);

                (*g).sweepgcopage = next;
                cost += steps as usize * GC_SWEEPPAGESTEPCOST as usize;
            }

            if (*g).sweepgcopage.is_null() {
                LUAU_ASSERT!(!crate::isdead!(g, (*g).mainthread as *mut GCObject));
                makewhite!(g, (*g).mainthread as *mut GCObject);

                shrinkbuffers(l);

                (*g).gcstate = GCSpause as u8;
            }
        }
        _ => {
            LUAU_ASSERT!(false);
        }
    }

    cost
}
