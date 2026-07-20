use crate::functions::enumedge::enumedge;
use crate::functions::enumnode::enumnode;
use crate::macros::gcvalue::gcvalue;
use crate::macros::iscollectable::iscollectable;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::type_aliases::up_val::UpVal;

#[allow(non_snake_case)]
pub(crate) unsafe fn enumupval(ctx: *mut EnumContext, uv: *mut UpVal) {
    enumnode(
        ctx,
        obj2gco!(uv as *mut GCObject),
        core::mem::size_of::<UpVal>(),
        core::ptr::null(),
    );

    if iscollectable!((*uv).v) {
        enumedge(
            ctx,
            obj2gco!(uv as *mut GCObject),
            gcvalue!((*uv).v),
            b"value\0" as *const _ as *const core::ffi::c_char,
        );
    }
}
