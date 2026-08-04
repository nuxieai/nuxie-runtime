use crate::functions::cleartable::cleartable;
use crate::functions::clearupvals::clearupvals;
use crate::functions::embeddermarkref::embeddermarkref;
use crate::functions::markmt::markmt;
use crate::functions::marktaggetmt::marktaggetmt;
use crate::functions::markudatadirectaccess::markudatadirectaccess;
use crate::functions::markudatadirectfields::markudatadirectfields;
use crate::functions::propagateall::propagateall;
use crate::functions::remarkupvals::remarkupvals;
use crate::macros::gc_satomic::GCSatomic;
use crate::macros::gc_satomic::GCSsweep;
use crate::macros::markobject::markobject;
use crate::macros::otherwhite::otherwhite;
use crate::records::gc_object::GCObject;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn atomic(l: *mut lua_State) -> usize {
    let g = (*l).global;
    LUAU_ASSERT!((*g).gcstate as i32 == GCSatomic);

    let mut work = 0usize;

    work += remarkupvals(g);
    work += propagateall(g);

    (*g).gray = (*g).weak;
    (*g).weak = core::ptr::null_mut();
    LUAU_ASSERT!(!crate::iswhite!((*g).mainthread as *mut GCObject));
    markobject!(g, l);
    markmt(g);

    if luaur_common::FFlag::LuauUdataMetatablePinned.get() {
        marktaggetmt(g);
    }

    if luaur_common::DFFlag::LuauGcMarkUdataAccess.get() {
        markudatadirectaccess(g);
    }

    if luaur_common::FFlag::LuauDirectFieldGet.get() {
        markudatadirectfields(g);
    }

    work += propagateall(g);

    (*g).gray = (*g).grayagain;
    (*g).grayagain = core::ptr::null_mut();
    work += propagateall(g);

    if luaur_common::FFlag::LuauGcTraceUdata.get() {
        #[cfg(feature = "luai_gcmetrics")]
        let mut embedder_start = crate::functions::lua_clock::lua_clock();

        if let Some(embeddergc) = (*g).embeddergc {
            embeddergc((*g).mainthread, Some(embeddermarkref));
            while !(*g).gray.is_null() {
                work += propagateall(g);
                embeddergc((*g).mainthread, Some(embeddermarkref));
            }
        }

        #[cfg(feature = "luai_gcmetrics")]
        {
            (*g).gcmetrics.currcycle.atomictimeembedder +=
                crate::functions::record_gc_delta_time::record_gc_delta_time(&mut embedder_start);
        }
    }

    work += cleartable(l, (*g).weak);
    (*g).weak = core::ptr::null_mut();

    work += clearupvals(l);

    (*g).currentwhite = otherwhite!(g) as u8;
    (*g).sweepgcopage = (*g).allgcopages;
    (*g).gcstate = GCSsweep as u8;

    work
}
