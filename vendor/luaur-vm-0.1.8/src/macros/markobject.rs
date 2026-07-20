//! Generated skeleton item.
//! Node: `cxx:Macro:Luau.VM:VM/src/lgc.cpp:149:markobject`
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
//!   - calls <- function markmt (VM/src/lgc.cpp)
//!   - calls <- function markroot (VM/src/lgc.cpp)
//!   - calls <- function atomic (VM/src/lgc.cpp)
//! - outgoing:
//!   - translates_to -> rust_item markobject

#[allow(non_snake_case)]
#[macro_export]
macro_rules! markobject {
    ($g:expr, $t:expr) => {
        // lgc.cpp:149 — obj2gco on a typed GC pointer is a plain cast in C
        if $crate::macros::iswhite::iswhite!($t as *mut $crate::records::gc_object::GCObject) {
            $crate::functions::reallymarkobject::reallymarkobject(
                $g,
                $t as *mut $crate::records::gc_object::GCObject,
            );
        }
    };
}

pub use markobject;
