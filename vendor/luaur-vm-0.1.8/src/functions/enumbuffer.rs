use crate::functions::enumnode::enumnode;
use crate::macros::sizebuffer::sizebuffer;
use crate::records::enum_context::EnumContext;
use crate::type_aliases::buffer::Buffer;

#[allow(non_snake_case)]
pub(crate) unsafe fn enumbuffer(ctx: *mut EnumContext, b: *mut Buffer) {
    // Buffer is a collectable object; cast it to a GCObject for enumnode.
    // We must avoid using `obj2gco!` here because it expects a proper GCObject
    // layout (via `.tt()`), and a raw cast of the Buffer pointer into `c_void`
    // breaks that assumption during macro expansion.
    let gco = b as *mut core::ffi::c_void;

    enumnode(
        ctx,
        gco as *mut crate::records::gc_object::GCObject,
        sizebuffer((*b).len as usize),
        core::ptr::null(),
    );
}
