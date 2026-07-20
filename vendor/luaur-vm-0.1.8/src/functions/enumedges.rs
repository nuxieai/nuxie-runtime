use crate::functions::enumedge::enumedge;
use crate::macros::gcvalue::gcvalue;
use crate::macros::iscollectable::iscollectable;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::type_aliases::t_value::TValue;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub(crate) unsafe fn enumedges(
    ctx: *mut EnumContext,
    from: *mut GCObject,
    data: *mut TValue,
    size: usize,
    edgename: *const c_char,
) {
    for i in 0..size {
        let val_ptr = data.add(i);
        if iscollectable!(val_ptr) {
            enumedge(ctx, from, gcvalue!(val_ptr), edgename);
        }
    }
}
