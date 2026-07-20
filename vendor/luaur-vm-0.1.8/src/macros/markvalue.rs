//! Generated skeleton item.
//! Node: `cxx:Macro:Luau.VM:VM/src/lgc.cpp:142:markvalue`
//! Source: `VM/src/lgc.cpp`
//! Graph edges:
//! - declared_by: source_file VM/src/lgc.cpp
//! - source_includes:
//!   - includes -> source_file VM/src/lgc.h
//!   - includes -> source_file VM/src/lobject.h
//!   - includes -> source_file VM/src/lstate.h
//!   - includes -> source_file VM/src/ltable.h
//!   - includes -> source_file VM/src/lfunc.h
//!   - includes -> source_file VM/src/lstring.h
//!   - includes -> source_file VM/src/ldo.h
//!   - includes -> source_file VM/src/lmem.h
//!   - includes -> source_file VM/src/ludata.h
//!   - includes -> source_file VM/src/lbuffer.h
//!   - includes -> source_file VM/src/lclass.h
//! - incoming:
//!   - declares <- source_file VM/src/lgc.cpp
//!   - calls <- function reallymarkobject (VM/src/lgc.cpp)
//!   - calls <- function traversetable (VM/src/lgc.cpp)
//!   - calls <- function traverseproto (VM/src/lgc.cpp)
//!   - calls <- function traverseclosure (VM/src/lgc.cpp)
//!   - calls <- function traversestack (VM/src/lgc.cpp)
//!   - calls <- function traverseclass (VM/src/lgc.cpp)
//!   - calls <- function traverseobject (VM/src/lgc.cpp)
//!   - calls <- function markroot (VM/src/lgc.cpp)
//!   - calls <- function remarkupvals (VM/src/lgc.cpp)
//! - outgoing:
//!   - translates_to -> rust_item markvalue

#[allow(non_snake_case)]
#[macro_export]
macro_rules! markvalue {
    ($g:expr, $o:expr) => {
        // lgc.cpp:142 — checkconsistency is a debug no-op in release builds
        if $crate::macros::iscollectable::iscollectable!($o)
            && $crate::macros::iswhite::iswhite!($crate::macros::gcvalue::gcvalue!($o))
        {
            $crate::functions::reallymarkobject::reallymarkobject(
                $g,
                $crate::macros::gcvalue::gcvalue!($o),
            );
        }
    };
}

pub use markvalue;
