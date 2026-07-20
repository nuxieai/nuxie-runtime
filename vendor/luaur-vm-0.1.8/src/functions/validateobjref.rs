use crate::macros::isdead::isdead;
use crate::macros::keepinvariant::keepinvariant;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn validateobjref(g: *mut global_State, f: *mut GCObject, t: *mut GCObject) {
    LUAU_ASSERT!(!isdead!(g, t));

    if keepinvariant(g) {
        // basic incremental invariant: black can't point to white
        const WHITE0BIT: u8 = 0;
        const WHITE1BIT: u8 = 1;
        const BLACKBIT: u8 = 2;

        const WHITEBITS: u8 = (1 << WHITE0BIT) | (1 << WHITE1BIT);
        const BLACKBIT_MASK: u8 = 1 << BLACKBIT;

        let is_black_f = ((*f).gch.marked & BLACKBIT_MASK) != 0;
        let is_white_t = ((*t).gch.marked & WHITEBITS) != 0;

        LUAU_ASSERT!(!(is_black_f && is_white_t));
    }
}
