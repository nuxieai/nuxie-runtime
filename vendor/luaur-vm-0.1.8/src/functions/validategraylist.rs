//! `validategraylist` — validate every node in a GC gray list.
//! C++ source: `VM/src/lgcdebug.cpp:218`
//!
//! Walks the singly-linked gray list starting at `o`; asserts each node is
//! still gray and follows the per-type `gclist` pointer to the next node.
//! Returns immediately if the GC invariant is not active (sweep phase etc.).

use crate::enums::lua_type::lua_Type;
use crate::macros::keepinvariant::keepinvariant;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn validategraylist(g: *mut global_State, mut o: *mut GCObject) {
    if !keepinvariant(g) {
        return;
    }

    while !o.is_null() {
        LUAU_ASSERT!(crate::isgray!(o));

        match (*o).gch.tt as i32 {
            t if t == lua_Type::LUA_TTABLE as i32 => {
                o = (*crate::gco2h!(o)).gclist;
            }
            t if t == lua_Type::LUA_TFUNCTION as i32 => {
                o = (*crate::gco2cl!(o)).gclist;
            }
            t if t == lua_Type::LUA_TTHREAD as i32 => {
                o = (*crate::gco2th!(o)).gclist;
            }
            t if t == lua_Type::LUA_TCLASS as i32 => {
                o = (*crate::gco2class!(o)).gclist;
            }
            t if t == lua_Type::LUA_TOBJECT as i32 => {
                o = (*crate::gco2object!(o)).gclist;
            }
            t if t == lua_Type::LUA_TPROTO as i32 => {
                o = (*crate::gco2p!(o)).gclist;
            }
            _ => {
                LUAU_ASSERT!(false);
                return;
            }
        }
    }
}
