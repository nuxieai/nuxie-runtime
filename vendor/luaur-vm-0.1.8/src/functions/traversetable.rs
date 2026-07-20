//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:321:traversetable`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:321-366, hand-ported)

use crate::enums::lua_type::lua_Type;
use crate::functions::gettablemode::gettablemode;
use crate::functions::removeentry::removeentry;
use crate::macros::gkey::{gkey, gval};
use crate::macros::gnode::gnode;
use crate::macros::markobject::markobject;
use crate::macros::markvalue::markvalue;
use crate::macros::sizenode::sizenode;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttype::ttype;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use crate::records::lua_table::LuaTable;
use core::ffi::c_int;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn traversetable(g: *mut global_State, h: *mut LuaTable) -> c_int {
    let mut weakkey: c_int = 0;
    let mut weakvalue: c_int = 0;
    if !(*h).metatable.is_null() {
        markobject!(g, (*h).metatable);
    }

    // is there a weak mode?
    let modev = gettablemode(g, h);
    if !modev.is_null() {
        // strchr(modev, 'k') / strchr(modev, 'v') as one scan over the mode string
        let mut p = modev;
        while *p != 0 {
            if *p as u8 == b'k' {
                weakkey = 1;
            }
            if *p as u8 == b'v' {
                weakvalue = 1;
            }
            p = p.add(1);
        }
        if weakkey != 0 || weakvalue != 0 {
            // is really weak?
            (*h).gclist = (*g).weak; // must be cleared after GC, ...
            (*g).weak = h as *mut GCObject; // ... so put in the appropriate list
        }
    }

    if weakkey != 0 && weakvalue != 0 {
        return 1;
    }
    if weakvalue == 0 {
        let mut i = (*h).sizearray;
        while i > 0 {
            i -= 1;
            markvalue!(g, (*h).array.add(i as usize));
        }
    }
    let mut i: c_int = sizenode!(h);
    while i > 0 {
        i -= 1;
        let n = gnode!(h, i);
        LUAU_ASSERT!(ttype!(gkey!(n)) != lua_Type::LUA_TDEADKEY as i32 || ttisnil!(gval!(n)));
        if ttisnil!(gval!(n)) {
            removeentry(n); // remove empty entries
        } else {
            LUAU_ASSERT!(!ttisnil!(gkey!(n)));
            if weakkey == 0 {
                markvalue!(g, gkey!(n));
            }
            if weakvalue == 0 {
                markvalue!(g, gval!(n));
            }
        }
    }
    (weakkey != 0 || weakvalue != 0) as c_int
}
