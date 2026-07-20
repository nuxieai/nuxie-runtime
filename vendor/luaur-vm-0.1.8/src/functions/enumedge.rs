use crate::functions::enumtopointer::enumtopointer;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use core::ffi::{c_char, c_void};

#[allow(non_snake_case)]
pub(crate) unsafe fn enumedge(
    ctx: *mut EnumContext,
    from: *mut GCObject,
    to: *mut GCObject,
    edgename: *const c_char,
) {
    let ctx_ref = &*ctx;
    if let Some(edge_fn) = ctx_ref.edge {
        edge_fn(
            ctx_ref.context,
            enumtopointer(from),
            enumtopointer(to),
            edgename,
        );
    }
}
