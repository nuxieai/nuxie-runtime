use crate::functions::enumedge::enumedge;
use crate::functions::enumedges::enumedges;
use crate::functions::enumnode::enumnode;
use crate::macros::clvalue::clvalue;
use crate::macros::getstr::getstr;
use crate::macros::lua_idsize::LUA_IDSIZE;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::call_info::CallInfo;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::records::lua_state::lua_State;
use crate::records::proto::Proto;
use crate::records::t_string::TString;
use crate::type_aliases::closure::Closure;
use crate::type_aliases::t_value::TValue;
use core::ffi::{c_char, c_int};

#[allow(non_snake_case)]
pub unsafe fn enumthread(ctx: *mut EnumContext, th: *mut lua_State) {
    let size = core::mem::size_of::<lua_State>()
        + core::mem::size_of::<TValue>() * (*th).stacksize as usize
        + core::mem::size_of::<CallInfo>() * (*th).size_ci as usize;

    let mut tcl: *mut Closure = core::ptr::null_mut();
    let mut ci: *mut CallInfo = (*th).base_ci;
    while ci <= (*th).ci {
        if ttisfunction!((*ci).func) {
            tcl = clvalue!((*ci).func);
            break;
        }
        ci = ci.wrapping_add(1);
    }

    if !tcl.is_null() && (*tcl).isC == 0 {
        let tcl_l = core::ptr::addr_of!((*tcl).inner.l).cast::<crate::records::closure::LClosure>();
        let p: *mut Proto = (*tcl_l).p;
        if !p.is_null() {
            let mut buf: [c_char; 256] = [0; 256];

            let src_str = if !(*p).source.is_null() {
                getstr((*p).source)
            } else {
                c"unnamed".as_ptr()
            };
            let debugname_str = if !(*p).debugname.is_null() {
                getstr((*p).debugname)
            } else {
                c"unnamed".as_ptr()
            };

            let _ = snprintf(
                buf.as_mut_ptr(),
                buf.len() as u32,
                c"thread at %s:%d %s".as_ptr(),
                debugname_str,
                (*p).linedefined,
                src_str,
            );

            enumnode(ctx, obj2gco!(th as *mut GCObject), size, buf.as_ptr());
        } else {
            enumnode(ctx, obj2gco!(th as *mut GCObject), size, core::ptr::null());
        }
    } else {
        enumnode(ctx, obj2gco!(th as *mut GCObject), size, core::ptr::null());
    }

    enumedge(
        ctx,
        obj2gco!(th as *mut GCObject),
        obj2gco!((*th).gt as *mut GCObject),
        c"globals".as_ptr(),
    );

    if (*th).top > (*th).stack {
        enumedges(
            ctx,
            obj2gco!(th as *mut GCObject),
            (*th).stack,
            (*th).top.offset_from((*th).stack) as usize,
            c"stack".as_ptr(),
        );
    }
}

// snprintf is not available in core::ffi, but luau-vm already provides it via lobject.h
extern "C" {
    fn snprintf(s: *mut c_char, n: u32, format: *const c_char, ...) -> c_int;
}
