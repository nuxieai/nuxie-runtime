use crate::functions::hashpointer::hashpointer;
use crate::functions::lua_a_toobject::luaO_nilobject;
use crate::macros::gkey::gkey;
use crate::macros::lightuserdatatag::lightuserdatatag;
use crate::macros::pvalue::pvalue;
use crate::macros::ttislightuserdata::ttislightuserdata;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_h_getp(t: *mut LuaTable, key: *mut core::ffi::c_void, tag: i32) -> *const TValue {
    let mut n: *mut LuaNode = hashpointer(t as *const _, key);
    loop {
        let nk = gkey!(n);
        if ttislightuserdata!(nk) && pvalue!(nk) == key && lightuserdatatag!(nk) == tag {
            return &(*n).val;
        }
        let next_offset = (*n).key.next();
        if next_offset == 0 {
            break;
        }
        n = n.offset(next_offset as isize);
    }
    luaO_nilobject
}

pub use lua_h_getp as luaH_getp;
