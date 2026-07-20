//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:839:remarkupvals`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:839-856, hand-ported)

use crate::macros::isblack::isblack;
use crate::macros::isgray::isgray;
use crate::macros::markvalue::markvalue;
use crate::macros::upisopen::upisopen;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use crate::records::up_val::UpVal;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn remarkupvals(g: *mut global_State) -> usize {
    let mut work: usize = 0;

    let uvhead = core::ptr::addr_of_mut!((*g).uvhead);
    let mut uv = (*g).uvhead.u.open.next;
    while uv != uvhead {
        work += core::mem::size_of::<UpVal>();

        LUAU_ASSERT!(upisopen!(uv));
        LUAU_ASSERT!(
            (*(*uv).u.open.next).u.open.prev == uv && (*(*uv).u.open.prev).u.open.next == uv
        );
        // open upvalues are never black
        LUAU_ASSERT!(!isblack!(uv as *mut GCObject));

        if isgray!(uv as *mut GCObject) {
            markvalue!(g, (*uv).v);
        }

        uv = (*uv).u.open.next;
    }

    work
}
