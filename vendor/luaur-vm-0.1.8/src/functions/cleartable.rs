use crate::functions::gettablemode::gettablemode;
use crate::functions::removeentry::removeentry;
use crate::functions::tableresizeprotected::tableresizeprotected;
use crate::macros::gkey::{gkey, gval};
use crate::macros::gnode::gnode;
use crate::macros::iscleared::iscleared;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::sizenode::sizenode;
use crate::macros::ttisnil::ttisnil;
use crate::records::gc_object::GCObject;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[inline]
unsafe fn contains_s(mut mode: *const core::ffi::c_char) -> bool {
    while !mode.is_null() && *mode != 0 {
        if *mode == b's' as core::ffi::c_char {
            return true;
        }
        mode = mode.add(1);
    }
    false
}

#[allow(non_snake_case)]
pub unsafe fn cleartable(l: *mut lua_State, mut list: *mut GCObject) -> usize {
    let mut work = 0usize;

    while !list.is_null() {
        let h = list as *mut LuaTable;
        let hsize = sizenode!(h);
        work += core::mem::size_of::<LuaTable>()
            + core::mem::size_of::<TValue>() * (*h).sizearray as usize
            + core::mem::size_of::<LuaNode>() * hsize as usize;

        let mut i = (*h).sizearray;
        while i > 0 {
            i -= 1;
            let o = (*h).array.add(i as usize);
            if iscleared!(o) {
                setnilvalue!(o);
            }
        }

        i = hsize;
        let mut activevalues = 0;
        while i > 0 {
            i -= 1;
            let n = gnode!(h, i);
            if !ttisnil!(gval!(n)) {
                if iscleared!(gkey!(n)) || iscleared!(gval!(n)) {
                    setnilvalue!(gval!(n));
                    removeentry(n);
                } else {
                    activevalues += 1;
                }
            }
        }

        let modev = gettablemode((*l).global, h);
        if !modev.is_null() && contains_s(modev) && activevalues < hsize * 3 / 8 {
            tableresizeprotected(l, h, activevalues);
        }

        list = (*h).gclist;
    }

    work
}
