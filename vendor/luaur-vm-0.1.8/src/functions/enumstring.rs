use crate::functions::enumnode::enumnode;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::sizestring::sizestring;
use crate::records::enum_context::EnumContext;
use crate::type_aliases::t_string::TString;

#[allow(non_snake_case)]
pub(crate) unsafe fn enumstring(ctx: *mut EnumContext, ts: *mut TString) {
    enumnode(
        ctx,
        obj2gco!(ts),
        sizestring((*ts).len as usize),
        core::ptr::null(),
    );
}
