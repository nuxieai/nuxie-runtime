use crate::functions::enumtopointer::enumtopointer;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use core::ffi::{c_char, c_void};

#[allow(non_snake_case)]
pub(crate) unsafe fn enumnode(
    ctx: *mut EnumContext,
    gco: *mut GCObject,
    size: usize,
    objname: *const c_char,
) {
    let ctx_ref = &*ctx;
    if let Some(node_fn) = ctx_ref.node {
        node_fn(
            ctx_ref.context,
            enumtopointer(gco),
            (*gco).gch.tt,
            (*gco).gch.memcat,
            size,
            objname,
        );
    }
}
