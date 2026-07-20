//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:858:clearupvals`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:858-890, hand-ported)

use crate::functions::lua_f_closeupval::luaF_closeupval;
use crate::macros::gcvalue::gcvalue;
use crate::macros::isblack::isblack;
use crate::macros::iscollectable::iscollectable;
use crate::macros::isgray::isgray;
use crate::macros::iswhite::iswhite;
use crate::macros::upisopen::upisopen;
use crate::records::gc_object::GCObject;
use crate::records::up_val::UpVal;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn clearupvals(l: *mut lua_State) -> usize {
    let g = (*l).global;

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
        LUAU_ASSERT!(
            iswhite!(uv as *mut GCObject)
                || !iscollectable!((*uv).v)
                || !iswhite!(gcvalue!((*uv).v))
        );

        if (*uv).markedopen != 0 {
            // upvalue is still open (belongs to alive thread)
            LUAU_ASSERT!(isgray!(uv as *mut GCObject));
            (*uv).markedopen = 0; // for next cycle
            uv = (*uv).u.open.next;
        } else {
            // upvalue is either dead, or alive but the thread is dead; unlink and close
            let next = (*uv).u.open.next;
            luaF_closeupval(l, uv, /* dead= */ iswhite!(uv as *mut GCObject));
            uv = next;
        }
    }

    work
}
