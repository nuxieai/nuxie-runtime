//! Generated skeleton item.
//! Node: `cxx:Function:Luau.VM:VM/src/lbuiltins.cpp:785:luau_f_type`
//! Source: `VM/src/lbuiltins.cpp`
//! Graph edges:
//! - declared_by: source_file VM/src/lbuiltins.cpp
//! - source_includes:
//!   - includes -> source_file VM/src/lbuiltins.h
//!   - includes -> source_file VM/src/lstate.h
//!   - includes -> source_file VM/src/lstring.h
//!   - includes -> source_file VM/src/ltable.h
//!   - includes -> source_file VM/src/lgc.h
//!   - includes -> source_file VM/src/lnumutils.h
//!   - includes -> source_file VM/src/ldo.h
//!   - includes -> source_file VM/src/lbuffer.h
//! - incoming:
//!   - declares <- source_file VM/src/lbuiltins.cpp
//! - outgoing:
//!   - type_ref -> type_alias StkId (VM/src/lobject.h)
//!   - type_ref -> type_alias TValue (VM/src/lobject.h)
//!   - calls -> macro ttype (VM/src/lobject.h)
//!   - calls -> macro setsvalue (VM/src/lobject.h)
//!   - translates_to -> rust_item luauF_type

use crate::macros::setsvalue::setsvalue;
use crate::macros::ttype::ttype;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_type(
    l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 {
        let tt = ttype!(arg0);
        let ttname = (*(*l).global).ttname[tt as usize];
        setsvalue!(l, res, ttname);
        return 1;
    }

    -1
}
