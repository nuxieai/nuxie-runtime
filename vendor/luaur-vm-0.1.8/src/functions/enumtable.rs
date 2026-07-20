use crate::functions::enumedge::enumedge;
use crate::functions::enumedges::enumedges;
use crate::functions::enumnode::enumnode;
use crate::macros::dummynode::dummynode;
use crate::macros::gcvalue::gcvalue;
use crate::macros::getstr::getstr;
use crate::macros::gfasttm::gfasttm;
use crate::macros::hvalue::hvalue;
use crate::macros::iscollectable::iscollectable;
use crate::macros::nvalue::nvalue;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::registry::registry;
use crate::macros::sizenode::sizenode;
use crate::macros::svalue::svalue;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisstring::ttisstring;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::records::lua_node::LuaNode;
use crate::records::lua_t_value::TValue;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_table::LuaTable as LuaTableAlias;
use crate::type_aliases::t_value::TValue as TValueAlias;
use crate::type_aliases::tms::TMS;
use core::ffi::{c_char, c_int};
use core::ptr;

#[allow(non_snake_case)]
pub(crate) unsafe fn enumtable(ctx: *mut EnumContext, h: *mut LuaTable) {
    let size = core::mem::size_of::<LuaTable>()
        + if (*h).node == dummynode as *mut LuaNode {
            0
        } else {
            sizenode!(h) as usize * core::mem::size_of::<LuaNode>()
        }
        + (*h).sizearray as usize * core::mem::size_of::<TValue>();

    let obj = obj2gco!(h);

    let h_ref = &*h;
    let registry_ptr = registry!((*ctx).L);
    let is_registry =
        h == hvalue!(core::ptr::addr_of!(*registry_ptr) as *mut TValue) as *mut LuaTable;

    enumnode(
        ctx,
        obj,
        size,
        if is_registry {
            b"registry\0".as_ptr() as *const c_char
        } else {
            ptr::null()
        },
    );

    if (*h).node != dummynode as *mut LuaNode {
        let mut weakkey = false;
        let mut weakvalue = false;

        let g = (*(*ctx).L).global;
        let metatable = (*h).metatable;
        if !metatable.is_null() {
            let mode = gfasttm(g, metatable, TMS::TM_MODE as i32);
            if !mode.is_null() && ttisstring!(mode) {
                let mode_str = svalue!(mode);
                let mode_slice = core::ffi::CStr::from_ptr(mode_str).to_bytes();
                weakkey = mode_slice.contains(&b'k');
                weakvalue = mode_slice.contains(&b'v');
            }
        }

        let node_count = sizenode!(h) as i32;
        for i in 0..node_count {
            let n = &(*(*h).node.add(i as usize));

            if !ttisnil!(&n.val) && (iscollectable!(&n.key) || iscollectable!(&n.val)) {
                if !weakkey && iscollectable!(&n.key) {
                    enumedge(
                        ctx,
                        obj,
                        gcvalue!(&n.key),
                        b"[key]\0".as_ptr() as *const c_char,
                    );
                }

                if !weakvalue && iscollectable!(&n.val) {
                    if ttisstring!(&n.key) {
                        enumedge(ctx, obj, gcvalue!(&n.val), svalue!(&n.key));
                    } else if ttisnumber!(&n.key) {
                        let mut buf = [0i8; 32];
                        let nvalue_ptr = nvalue!(&n.key);
                        extern "C" {
                            fn snprintf(
                                s: *mut c_char,
                                n: usize,
                                format: *const c_char,
                                ...
                            ) -> c_int;
                        }
                        snprintf(
                            buf.as_mut_ptr(),
                            buf.len(),
                            b"%.14g\0".as_ptr() as *const c_char,
                            nvalue_ptr,
                        );
                        enumedge(ctx, obj, gcvalue!(&n.val), buf.as_ptr());
                    } else {
                        let mut buf = [0i8; 32];
                        let tt = n.key.tt();
                        let global = (*(*ctx).L).global;
                        let ttname_ptr = (*global).ttname.as_ptr().add(tt as usize);
                        let name = getstr(ttname_ptr as *const crate::records::t_string::TString);
                        extern "C" {
                            fn snprintf(
                                s: *mut c_char,
                                n: usize,
                                format: *const c_char,
                                ...
                            ) -> c_int;
                        }
                        snprintf(
                            buf.as_mut_ptr(),
                            buf.len(),
                            b"[%s]\0".as_ptr() as *const c_char,
                            name,
                        );
                        enumedge(ctx, obj, gcvalue!(&n.val), buf.as_ptr());
                    }
                }
            }
        }
    }

    if (*h).sizearray > 0 {
        enumedges(
            ctx,
            obj,
            (*h).array,
            (*h).sizearray as usize,
            b"array\0".as_ptr() as *const c_char,
        );
    }

    if !(*h).metatable.is_null() {
        enumedge(
            ctx,
            obj,
            obj2gco!((*h).metatable),
            b"metatable\0".as_ptr() as *const c_char,
        );
    }
}
