//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:311:gettablemode`
//! Source: `VM/src/lgc.cpp:311-319` (hand-ported)

use crate::macros::gfasttm::gfasttm;
use crate::macros::svalue::svalue;
use crate::macros::ttisstring::ttisstring;
use crate::records::global_state::global_State;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::tms::TMS;

#[allow(non_snake_case)]
pub unsafe fn gettablemode(g: *mut global_State, h: *mut LuaTable) -> *const core::ffi::c_char {
    let mode = gfasttm(g, (*h).metatable, TMS::TM_MODE as i32);
    if !mode.is_null() && ttisstring!(mode) {
        return svalue!(mode);
    }
    core::ptr::null()
}
