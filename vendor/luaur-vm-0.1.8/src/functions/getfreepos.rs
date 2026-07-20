use crate::enums::lua_type::lua_Type;
use crate::macros::gkey::gkey;
use crate::macros::gnode::gnode;
use crate::macros::ttisnil::ttisnil;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;

pub(crate) unsafe fn getfreepos(t: *mut LuaTable) -> *mut LuaNode {
    // In the C++ source, lastfree is accessed as t->lastfree.
    // In the Rust LuaTable record, lastfree is part of the union.
    // We access it through the union field.
    while (*t).union.lastfree > 0 {
        (*t).union.lastfree -= 1;

        let n = gnode!(t, (*t).union.lastfree);
        if ttisnil!(gkey!(n)) {
            return n;
        }
    }
    core::ptr::null_mut() // could not find a free place
}
