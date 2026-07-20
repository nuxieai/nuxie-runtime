use crate::functions::validateobjref::validateobjref;
use crate::macros::gcvalue::gcvalue;
use crate::macros::iscollectable::iscollectable;
use crate::macros::ttype::ttype;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn validateref(g: *mut global_State, f: *mut GCObject, v: *mut TValue) {
    if iscollectable!(v) {
        LUAU_ASSERT!(ttype!(v) == (*gcvalue!(v)).gch.tt as i32);
        validateobjref(g, f, gcvalue!(v));
    }
}
