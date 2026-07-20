use crate::functions::dumpref::dumpref;
use crate::functions::dumprefs::dumprefs;
use crate::macros::dummynode::dummynode;
use crate::macros::gcvalue::gcvalue;
use crate::macros::iscollectable::iscollectable;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::sizenode::sizenode;
use crate::macros::ttisnil::ttisnil;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumptable(f: *mut core::ffi::c_void, h: *mut LuaTable) {
    let h_ref = &*h;

    let size = core::mem::size_of::<LuaTable>()
        + if h_ref.node == dummynode as *mut LuaNode {
            0
        } else {
            sizenode!(h) as usize * core::mem::size_of::<LuaNode>()
        }
        + h_ref.sizearray as usize * core::mem::size_of::<TValue>();

    extern "C" {
        fn fprintf(
            stream: *mut core::ffi::c_void,
            format: *const core::ffi::c_char,
            ...
        ) -> core::ffi::c_int;
    }

    extern "C" {
        fn fputc(c: core::ffi::c_int, stream: *mut core::ffi::c_void) -> core::ffi::c_int;
    }

    fprintf(
        f,
        b"{\"type\":\"table\",\"cat\":%d,\"size\":%d\0".as_ptr() as *const core::ffi::c_char,
        h_ref.memcat as core::ffi::c_int,
        size as core::ffi::c_int,
    );

    if h_ref.node != dummynode as *mut LuaNode {
        fprintf(f, b",\"pairs\":[\0".as_ptr() as *const core::ffi::c_char);

        let mut first = true;

        for i in 0..sizenode!(h) {
            let node_ptr = h_ref.node.add(i as usize);
            let n = &*node_ptr;

            if !ttisnil!(&n.val) && (iscollectable!(&n.key) || iscollectable!(&n.val)) {
                if !first {
                    fputc(',' as core::ffi::c_int, f);
                }
                first = false;

                if iscollectable!(&n.key) {
                    dumpref(f, gcvalue!(&n.key));
                } else {
                    fprintf(f, b"null\0".as_ptr() as *const core::ffi::c_char);
                }

                fputc(',' as core::ffi::c_int, f);

                if iscollectable!(&n.val) {
                    dumpref(f, gcvalue!(&n.val));
                } else {
                    fprintf(f, b"null\0".as_ptr() as *const core::ffi::c_char);
                }
            }
        }

        fprintf(f, b"]\0".as_ptr() as *const core::ffi::c_char);
    }

    if h_ref.sizearray != 0 {
        fprintf(f, b",\"array\":[\0".as_ptr() as *const core::ffi::c_char);
        dumprefs(f, h_ref.array, h_ref.sizearray as usize);
        fprintf(f, b"]\0".as_ptr() as *const core::ffi::c_char);
    }

    if !h_ref.metatable.is_null() {
        fprintf(f, b",\"metatable\":\0".as_ptr() as *const core::ffi::c_char);
        dumpref(f, obj2gco!(h_ref.metatable));
    }

    fprintf(f, b"}\0".as_ptr() as *const core::ffi::c_char);
}
