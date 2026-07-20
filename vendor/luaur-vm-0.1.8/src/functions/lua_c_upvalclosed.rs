//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:1327:lua_c_upvalclosed`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:1327-1347, hand-ported)

use crate::macros::gc_spause::GCSpause;
use crate::macros::gray_2_black::gray2black;
use crate::macros::isgray::isgray;
use crate::macros::keepinvariant::keepinvariant;
use crate::macros::lua_c_barrier::luaC_barrier;
use crate::macros::makewhite::makewhite;
use crate::macros::upisopen::upisopen;
use crate::records::gc_object::GCObject;
use crate::records::up_val::UpVal;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaC_upvalclosed(l: *mut lua_State, uv: *mut UpVal) {
    let g = (*l).global;
    let o = uv as *mut GCObject;

    LUAU_ASSERT!(!upisopen!(uv)); // upvalue was closed but needs GC state fixup

    if isgray!(o) {
        if keepinvariant(g) {
            gray2black!(o); // closed upvalues need barrier
            luaC_barrier!(l, uv, (*uv).v);
        } else {
            // sweep phase: sweep it (turning it into white)
            makewhite!(g, o);
            LUAU_ASSERT!((*g).gcstate as i32 != GCSpause);
        }
    }
}

#[allow(unused_imports)]
pub use luaC_upvalclosed as lua_c_upvalclosed;
