use crate::functions::enumedge::enumedge;
use crate::functions::enumedges::enumedges;
use crate::functions::enumnode::enumnode;
use crate::functions::enumtopointer::enumtopointer;
use crate::macros::getstr::getstr;
use crate::macros::lua_idsize::LUA_IDSIZE;
use crate::macros::lua_tnone::LUA_TNONE;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::records::proto::Proto;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::loc_var::LocVar;
use crate::type_aliases::t_string::TString;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub(crate) unsafe fn enumproto(ctx: *mut EnumContext, p: *mut Proto) {
    let p_ref = &*p;

    let size = core::mem::size_of::<Proto>()
        + core::mem::size_of::<Instruction>() * p_ref.sizecode as usize
        + core::mem::size_of::<*mut Proto>() * p_ref.sizep as usize
        + core::mem::size_of::<TValue>() * p_ref.sizek as usize
        + p_ref.sizelineinfo as usize
        + core::mem::size_of::<LocVar>() * p_ref.sizelocvars as usize
        + core::mem::size_of::<*mut TString>() * p_ref.sizeupvalues as usize;

    let ctx_ref = &*ctx;

    // Manual expansion of obj2gco for Proto because Proto is not a union member of GCObject
    // and does not have a .tt() method, but its hdr (GCheader) is at offset 0.
    let p_gco = p as *mut GCObject;

    if !p_ref.execdata.is_null() {
        let global = (*ctx_ref.L).global;
        if let Some(getmemorysize) = (*global).ecb.getmemorysize {
            let nativesize = getmemorysize(ctx_ref.L, p);

            if let Some(node_cb) = ctx_ref.node {
                node_cb(
                    ctx_ref.context,
                    p_ref.execdata,
                    LUA_TNONE as u8,
                    p_ref.hdr.memcat,
                    nativesize,
                    core::ptr::null(),
                );
            }

            if let Some(edge_cb) = ctx_ref.edge {
                edge_cb(
                    ctx_ref.context,
                    enumtopointer(p_gco),
                    p_ref.execdata,
                    c"[native]".as_ptr(),
                );
            }
        }
    }

    let mut buf = [0 as core::ffi::c_char; LUA_IDSIZE as usize];

    let debugname = if !p_ref.debugname.is_null() {
        getstr(p_ref.debugname)
    } else {
        c"unnamed".as_ptr()
    };

    extern "C" {
        fn snprintf(
            s: *mut core::ffi::c_char,
            n: usize,
            format: *const core::ffi::c_char,
            ...
        ) -> core::ffi::c_int;
    }

    if !p_ref.source.is_null() {
        snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"proto %s:%d %s".as_ptr(),
            debugname,
            p_ref.linedefined,
            getstr(p_ref.source),
        );
    } else {
        snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"proto %s:%d".as_ptr(),
            debugname,
            p_ref.linedefined,
        );
    }

    enumnode(ctx, p_gco, size, buf.as_ptr());

    if p_ref.sizek > 0 {
        enumedges(
            ctx,
            p_gco,
            p_ref.k,
            p_ref.sizek as usize,
            c"constants".as_ptr(),
        );
    }

    for i in 0..p_ref.sizep {
        let sub_proto = *p_ref.p.add(i as usize);
        enumedge(ctx, p_gco, sub_proto as *mut GCObject, c"protos".as_ptr());
    }
}
