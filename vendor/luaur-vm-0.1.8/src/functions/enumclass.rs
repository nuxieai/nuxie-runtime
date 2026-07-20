use crate::functions::enumedge::enumedge;
use crate::functions::enumnode::enumnode;
use crate::macros::gcvalue::gcvalue;
use crate::macros::getstr::getstr;
use crate::macros::iscollectable::iscollectable;
use crate::macros::lua_idsize::LUA_IDSIZE;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::records::luau_class::LuauClass;
use core::ffi::{c_char, c_int};

#[allow(non_snake_case)]
pub(crate) unsafe fn enumclass(ctx: *mut EnumContext, lco: *mut LuauClass) {
    let lco_ref = &*lco;
    let mut buf = [0i8; LUA_IDSIZE as usize];
    let obj = lco as *mut GCObject;

    extern "C" {
        fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
    }

    snprintf(
        buf.as_mut_ptr(),
        buf.len(),
        b"class object %s\0".as_ptr() as *const c_char,
        getstr(lco_ref.name),
    );

    enumnode(ctx, obj, core::mem::size_of::<LuauClass>(), buf.as_ptr());
    enumedge(
        ctx,
        obj,
        lco_ref.name as *mut GCObject,
        b"classname\0".as_ptr() as *const c_char,
    );
    enumedge(
        ctx,
        obj,
        lco_ref.memberstooffset as *mut GCObject,
        b"classoffsets\0".as_ptr() as *const c_char,
    );

    let numberofstaticmembers = lco_ref.numberofallmembers - lco_ref.numberofinstancemembers;
    for i in 0..numberofstaticmembers {
        let val_ptr = lco_ref.staticmembers.add(i as usize);
        if !iscollectable!(val_ptr) {
            continue;
        }

        let mut membername = [0i8; 32];
        let name_ptr = *lco_ref
            .offsettomember
            .add((i + lco_ref.numberofinstancemembers) as usize);
        snprintf(
            membername.as_mut_ptr(),
            membername.len(),
            b"%s\0".as_ptr() as *const c_char,
            getstr(name_ptr),
        );
        enumedge(ctx, obj, gcvalue!(val_ptr), membername.as_ptr());
    }

    for i in 0..lco_ref.numberofallmembers {
        enumedge(
            ctx,
            obj,
            *lco_ref.offsettomember.add(i as usize) as *mut GCObject,
            b"membername\0".as_ptr() as *const c_char,
        );
    }

    enumedge(
        ctx,
        obj,
        lco_ref.metatable as *mut GCObject,
        b"metatable\0".as_ptr() as *const c_char,
    );
}
