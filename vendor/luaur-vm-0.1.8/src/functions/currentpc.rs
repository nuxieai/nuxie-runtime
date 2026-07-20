//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:17:currentpc`
//! Source: `VM/src/ldebug.cpp`
//! Graph edges:
//! - declared_by: source_file VM/src/ldebug.cpp
//! - source_includes:
//!   - includes -> source_file VM/src/ldebug.h
//!   - includes -> source_file VM/src/lapi.h
//!   - includes -> source_file VM/src/lfunc.h
//!   - includes -> source_file VM/src/lmem.h
//!   - includes -> source_file VM/src/lgc.h
//!   - includes -> source_file VM/src/ldo.h
//!   - includes -> source_file VM/src/lbytecode.h
//!   - includes -> source_file VM/src/lstring.h
//! - incoming:
//!   - declares <- source_file VM/src/ldebug.cpp
//!   - calls <- function currentline (VM/src/ldebug.cpp)
//!   - calls <- function lua_getlocal (VM/src/ldebug.cpp)
//!   - calls <- function lua_setlocal (VM/src/ldebug.cpp)
//! - outgoing:
//!   - calls -> macro pcRel (VM/src/ldebug.h)
//!   - calls -> macro ci_func (VM/src/lstate.h)
//!   - translates_to -> rust_item currentpc

use crate::enums::lua_type::lua_Type;
use crate::macros::ci_func::ci_func;
use crate::macros::pc_rel::pcRel;
use crate::records::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;

pub(crate) unsafe fn currentpc(_l: *mut lua_State, ci: *mut CallInfo) -> core::ffi::c_int {
    let cl = ci_func!(ci);
    let lcl = core::ptr::addr_of!((*cl).inner.l).cast::<crate::records::closure::LClosure>();
    pcRel!((*ci).savedpc, (*lcl).p)
}
