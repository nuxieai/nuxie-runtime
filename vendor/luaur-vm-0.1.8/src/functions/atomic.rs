use crate::functions::cleartable::cleartable;
use crate::functions::clearupvals::clearupvals;
use crate::functions::markmt::markmt;
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
    work += propagateall(g);

    (*g).gray = (*g).grayagain;
    (*g).grayagain = core::ptr::null_mut();
    work += propagateall(g);

    work += cleartable(l, (*g).weak);
    (*g).weak = core::ptr::null_mut();

    work += clearupvals(l);

    (*g).currentwhite = otherwhite!(g) as u8;
    (*g).sweepgcopage = (*g).allgcopages;
    (*g).gcstate = GCSsweep as u8;

    work
}
