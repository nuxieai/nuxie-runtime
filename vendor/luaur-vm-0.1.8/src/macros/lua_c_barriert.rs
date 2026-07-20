//! Generated skeleton item.
//! Node: `cxx:Macro:Luau.VM:VM/src/lgc.h:97:lua_c_barriert`
//! Source: `VM/src/lgc.h`
//! Graph edges:
//! - declared_by: source_file VM/src/lgc.h
//! - source_includes:
//!   - includes -> source_file VM/src/ldo.h
//!   - includes -> source_file VM/src/lobject.h
//!   - includes -> source_file VM/src/lstate.h
//! - incoming:
//!   - declares <- source_file VM/src/lgc.h
//!   - calls <- function executeSETTABLEKS (CodeGen/src/CodeGenUtils.cpp)
//!   - calls <- function lua_rawsetfield (VM/src/lapi.cpp)
//!   - calls <- function lua_rawset (VM/src/lapi.cpp)
//!   - calls <- function lua_rawseti (VM/src/lapi.cpp)
//!   - calls <- function lua_rawsetptagged (VM/src/lapi.cpp)
//!   - calls <- function lua_ref (VM/src/lapi.cpp)
//!   - calls <- function luauF_rawset (VM/src/lbuiltins.cpp)
//!   - calls <- function luauF_tinsert (VM/src/lbuiltins.cpp)
//!   - calls <- function newkey (VM/src/ltable.cpp)
//!   - calls <- function VM_CASE (VM/src/lvmexecute.cpp)
//!   - calls <- function VM_CASE (VM/src/lvmexecute.cpp)
//!   - calls <- function VM_CASE (VM/src/lvmexecute.cpp)
//!   - calls <- function VM_CASE (VM/src/lvmexecute.cpp)
//!   - calls <- function loadsafe (VM/src/lvmload.cpp)
//!   - calls <- function luaV_settable (VM/src/lvmutils.cpp)
//! - outgoing:
//!   - translates_to -> rust_item luaC_barriert

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaC_barriert {
    ($L:expr, $t:expr, $v:expr) => {
        // lgc.h:97
        if $crate::macros::iscollectable::iscollectable!($v)
            && $crate::macros::isblack::isblack!($t as *mut $crate::records::gc_object::GCObject)
            && $crate::macros::iswhite::iswhite!($crate::macros::gcvalue::gcvalue!($v))
        {
            $crate::functions::lua_c_barriertable::luaC_barriertable(
                $L,
                $t,
                $crate::macros::gcvalue::gcvalue!($v),
            );
        }
    };
}

pub use luaC_barriert;
