use crate::functions::enumedge::enumedge;
use crate::functions::enumnode::enumnode;
use crate::macros::gcvalue::gcvalue;
use crate::macros::getstr::getstr;
use crate::macros::iscollectable::iscollectable;
use crate::macros::lua_idsize::LUA_IDSIZE;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::records::luau_object::LuauObject;
use core::ffi::{c_char, c_int};

#[allow(non_snake_case)]
pub(crate) unsafe fn enumobject(ctx: *mut EnumContext, inst: *mut LuauObject) {
    let inst_ref = &*inst;
    let mut buf = [0i8; LUA_IDSIZE as usize];

    let obj = inst as *mut GCObject;

    extern "C" {
        fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
    }

    snprintf(
        buf.as_mut_ptr(),
        buf.len(),
        b"object %s\0".as_ptr() as *const c_char,
        getstr((*inst_ref.lclass).name),
    );

    enumnode(ctx, obj, core::mem::size_of::<LuauObject>(), buf.as_ptr());

    for i in 0..(*inst_ref.lclass).numberofinstancemembers {
        let val_ptr = inst_ref.members.add(i as usize);
        if !iscollectable!(val_ptr) {
            continue;
        }

        let mut membername = [0i8; 32];
        let name_ptr = *(*inst_ref.lclass).offsettomember.add(i as usize);
        snprintf(
            membername.as_mut_ptr(),
            membername.len(),
            b"%s\0".as_ptr() as *const c_char,
            getstr(name_ptr),
        );

        enumedge(ctx, obj, gcvalue!(val_ptr), membername.as_ptr());
    }
}
