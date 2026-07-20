use crate::functions::enumnode::enumnode;
use crate::macros::dummynode::dummynode;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::sizenode::sizenode;
use crate::macros::sizeudata::sizeudata;
use crate::macros::svalue::svalue;
use crate::macros::ttisstring::ttisstring;
use crate::records::enum_context::EnumContext;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::records::udata::Udata;
use crate::type_aliases::lua_node::LuaNode as LuaNodeAlias;
use crate::type_aliases::lua_table::LuaTable as LuaTableAlias;
use crate::type_aliases::udata::Udata as UdataAlias;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub unsafe fn enumudata(ctx: *mut EnumContext, u: *mut Udata) {
    let mut name: *const c_char = core::ptr::null();

    let h = (*u).metatable;
    if !h.is_null() {
        let h = h as *mut LuaTable;
        if (*h).node != dummynode as *mut LuaNode {
            let n = (*h).node;
            let size = sizenode!(h) as usize;
            for i in 0..size {
                let node_ptr = n.add(i);
                let node: &LuaNodeAlias = &*node_ptr;

                if ttisstring!(&node.key) && ttisstring!(&node.val) {
                    let key_str = unsafe { svalue!(&node.key) };
                    let val_str = unsafe { svalue!(&node.val) };

                    let key_cmp =
                        unsafe { core::ffi::CStr::from_ptr(key_str).to_str().unwrap_or("") };
                    if key_cmp == "__type" {
                        name = val_str;
                        break;
                    }
                }
            }
        }
    }

    let gco = obj2gco!(u as *mut Udata);
    enumnode(ctx, gco, sizeudata((*u).len as usize), name);

    if !(*u).metatable.is_null() {
        let metatable_gco = obj2gco!((*u).metatable as *mut Udata);
        enumedge(
            ctx,
            gco,
            metatable_gco,
            core::ffi::CStr::from_bytes_with_nul(b"metatable\0")
                .unwrap()
                .as_ptr(),
        );
    }
}

use crate::functions::enumedge::enumedge;
