use crate::functions::gcstep::gcstep;
use crate::functions::markroot::markroot;
use crate::functions::shrinkbuffersfull::shrinkbuffersfull;
use crate::macros::gc_satomic::GCSsweep;
use crate::macros::gc_spause::GCSpause;
use crate::macros::keepinvariant::keepinvariant;
use crate::macros::upisopen::upisopen;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_c_fullgc(l: *mut lua_State) {
    let g = (*l).global;

    if keepinvariant(g) {
        (*g).sweepgcopage = (*g).allgcopages;
        (*g).gray = core::ptr::null_mut();
        (*g).grayagain = core::ptr::null_mut();
        (*g).weak = core::ptr::null_mut();
        (*g).gcstate = GCSsweep as u8;
    }

    LUAU_ASSERT!((*g).gcstate as i32 == GCSpause || (*g).gcstate as i32 == GCSsweep);
    while (*g).gcstate as i32 != GCSpause {
        LUAU_ASSERT!((*g).gcstate as i32 == GCSsweep);
        gcstep(l, usize::MAX);
    }

    let uvhead = core::ptr::addr_of_mut!((*g).uvhead);
    let mut uv = (*g).uvhead.u.open.next;
    while uv != uvhead {
        LUAU_ASSERT!(upisopen!(uv));
        (*uv).markedopen = 0;
        uv = (*uv).u.open.next;
    }

    markroot(l);
    while (*g).gcstate as i32 != GCSpause {
        gcstep(l, usize::MAX);
    }

    shrinkbuffersfull(l);

    let heapgoalsizebytes = ((*g).totalbytes / 100) * (*g).gcgoal as usize;
    (*g).GCthreshold = (*g).totalbytes * (((*g).gcgoal * (*g).gcstepmul / 100 - 100) as usize)
        / (*g).gcstepmul as usize;

    if (*g).GCthreshold < (*g).totalbytes {
        (*g).GCthreshold = (*g).totalbytes;
    }

    (*g).gcstats.heapgoalsizebytes = heapgoalsizebytes;
}

#[allow(unused_imports)]
pub use lua_c_fullgc as luaC_fullgc;
