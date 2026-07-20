//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:417:traversestack`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:417-430, hand-ported)

use crate::macros::markobject::markobject;
use crate::macros::markvalue::markvalue;
use crate::macros::stringmark::stringmark;
use crate::macros::upisopen::upisopen;
use crate::records::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn traversestack(g: *mut global_State, l: *mut lua_State) {
    markobject!(g, (*l).gt);
    if !(*l).namecall.is_null() {
        stringmark!((*l).namecall);
    }
    let mut o = (*l).stack;
    while o < (*l).top {
        markvalue!(g, o);
        o = o.add(1);
    }
    let mut uv = (*l).openupval;
    while !uv.is_null() {
        LUAU_ASSERT!(upisopen!(uv));
        (*uv).markedopen = 1;
        markobject!(g, uv);
        uv = (*uv).u.open.threadnext;
    }
}
