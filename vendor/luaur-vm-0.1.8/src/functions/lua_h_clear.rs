use crate::enums::lua_type::lua_Type;
use crate::macros::dummynode::dummynode;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::sizenode::sizenode;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;

#[inline]
unsafe fn maybesetaboundary(t: *mut LuaTable, boundary: core::ffi::c_int) {
    if (*t).union.aboundary <= 0 {
        (*t).union.aboundary = -boundary;
    }
}

#[allow(non_snake_case)]
pub unsafe fn lua_h_clear(tt: *mut LuaTable) {
    let mut i = 0;
    while i < (*tt).sizearray {
        setnilvalue!((*tt).array.add(i as usize));
        i += 1;
    }

    maybesetaboundary(tt, 0);

    if (*tt).node != dummynode as *mut LuaNode {
        let size = sizenode!(tt);
        (*tt).union.lastfree = size;

        let mut i = 0;
        while i < size {
            let n = (*tt).node.add(i as usize);
            (*n).key.value = Default::default();
            (*n).key.extra = [0];
            (*n).key.set_tt(lua_Type::LUA_TNIL as i32);
            setnilvalue!(core::ptr::addr_of_mut!((*n).val));
            (*n).key.set_next(0);
            i += 1;
        }
    }

    (*tt).tmcache = !0u8;
}

#[allow(unused_imports)]
pub use lua_h_clear as luaH_clear;
