//! Node: `cxx:Function:Luau.VM:VM/src/ltable.cpp:657:lua_h_getstr`
//! Source: `VM/src/ltable.cpp` (ltable.cpp:657-669, hand-ported)

use crate::macros::gkey::{gkey, gval};
use crate::macros::gnext::gnext;
use crate::macros::hashstr::hashstr;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisstring::ttisstring;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::records::t_string::TString;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luaH_getstr(t: *mut LuaTable, key: *mut TString) -> *const TValue {
    let mut n: *mut LuaNode = hashstr!(t, key);
    loop {
        // check whether `key' is somewhere in the chain
        if ttisstring!(gkey!(n)) && tsvalue!(gkey!(n)) == key {
            return gval!(n); // that's it
        }
        if gnext!(n) == 0 {
            break;
        }
        n = n.offset(gnext!(n) as isize);
    }
    luaO_nilobject
}

pub use luaH_getstr as lua_h_getstr;
