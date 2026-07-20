use crate::functions::enumedge::enumedge;
use crate::functions::enumedges::enumedges;
use crate::functions::enumnode::enumnode;
use crate::macros::getstr::getstr;
use crate::macros::lua_idsize::LUA_IDSIZE;
use crate::macros::size_cclosure::size_cclosure;
use crate::macros::size_lclosure::size_lclosure;
use crate::records::closure::Closure;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::records::proto::Proto;
use core::ffi::{c_char, c_int};

#[allow(non_snake_case)]
pub(crate) unsafe fn enumclosure(ctx: *mut EnumContext, cl: *mut Closure) {
    let cl_ref = &*cl;
    let obj = cl as *mut GCObject;

    extern "C" {
        fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
    }

    if cl_ref.isC != 0 {
        enumnode(
            ctx,
            obj,
            size_cclosure(cl_ref.nupvalues as c_int),
            cl_ref.inner.c.debugname,
        );
    } else {
        let p: *mut Proto = cl_ref.inner.l.p;
        let mut buf = [0i8; LUA_IDSIZE as usize];

        let unnamed = c"unnamed".as_ptr();
        let debug_name = if !(*p).debugname.is_null() {
            getstr((*p).debugname)
        } else {
            unnamed
        };

        if !(*p).source.is_null() {
            snprintf(
                buf.as_mut_ptr(),
                buf.len(),
                c"%s:%d %s".as_ptr(),
                debug_name,
                (*p).linedefined,
                getstr((*p).source),
            );
        } else {
            snprintf(
                buf.as_mut_ptr(),
                buf.len(),
                c"%s:%d".as_ptr(),
                debug_name,
                (*p).linedefined,
            );
        }

        enumnode(
            ctx,
            obj,
            size_lclosure(cl_ref.nupvalues as usize),
            buf.as_ptr(),
        );
    }

    enumedge(ctx, obj, cl_ref.env as *mut GCObject, c"env".as_ptr());

    if cl_ref.isC != 0 {
        if cl_ref.nupvalues > 0 {
            enumedges(
                ctx,
                obj,
                cl_ref.inner.c.upvals.as_ptr() as *mut _,
                cl_ref.nupvalues as usize,
                c"upvalue".as_ptr(),
            );
        }
    } else {
        enumedge(
            ctx,
            obj,
            cl_ref.inner.l.p as *mut GCObject,
            c"proto".as_ptr(),
        );

        if cl_ref.nupvalues > 0 {
            enumedges(
                ctx,
                obj,
                cl_ref.inner.l.uprefs.as_ptr() as *mut _,
                cl_ref.nupvalues as usize,
                c"upvalue".as_ptr(),
            );
        }
    }
}
