//! Generated skeleton item.
//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:608:isobjcleared`
//! Source: `VM/src/lgc.cpp`
//! Graph edges:
//! - declared_by: source_file VM/src/lgc.cpp
//! - source_includes:
//!   - includes -> source_file VM/src/lgc.h
//!   - includes -> source_file VM/src/lobject.h
//!   - includes -> source_file VM/src/lstate.h
//!   - includes -> source_file VM/src/ltable.h
//!   - includes -> source_file VM/src/lfunc.h
//!   - includes -> source_file VM/src/lstring.h
//!   - includes -> source_file VM/src/ldo.h
//!   - includes -> source_file VM/src/lmem.h
//!   - includes -> source_file VM/src/ludata.h
//!   - includes -> source_file VM/src/lbuffer.h
//!   - includes -> source_file VM/src/lclass.h
//! - incoming:
//!   - declares <- source_file VM/src/lgc.cpp
//! - outgoing:
//!   - calls -> macro stringmark (VM/src/lgc.cpp)
//!   - calls -> macro iswhite (VM/src/lgc.h)
//!   - translates_to -> rust_item isobjcleared

/// Returns non-zero if the GC object `o` has been cleared (collected).
///
/// Strings are treated as values and are never considered cleared: their white
/// bits are reset so they will not be swept, and 0 is returned immediately.
/// For all other collectable types the function returns the `iswhite` result —
/// non-zero means the object's white bits are still set, i.e. it was not
/// reached during the mark phase and has been (or will be) collected.
///
/// C++ original: `static int isobjcleared(GCObject* o)` in VM/src/lgc.cpp:608
#[inline]
pub unsafe fn isobjcleared(o: *mut crate::records::gc_object::GCObject) -> i32 {
    use crate::enums::lua_type::lua_Type;

    if (*o).gch.tt == lua_Type::LUA_TSTRING as u8 {
        // strings are 'values', so they are never weak — stringmark(&o->ts)
        crate::stringmark!(
            core::ptr::addr_of_mut!((*o).ts) as *mut crate::records::t_string::TString
        );
        0
    } else {
        // iswhite(o): test WHITE0BIT (bit 0) and WHITE1BIT (bit 1)
        // Using inline bit test because the iswhite! macro has a type-mismatch
        // (testbits takes i32 but marked is u8, and WHITE0BIT/WHITE1BIT are not
        // re-exported from crate::macros in this workspace build).
        ((*o).gch.marked & 0b0000_0011_u8) as i32
    }
}
