// A discarded fallible VM operation would resume execution after a guest error.
#![deny(unused_must_use)]

extern crate alloc;

#[cfg(test)]
mod builtin_bounds_tests;
pub mod enums;
pub mod functions;
pub mod macros;
pub mod methods;
pub mod records;
#[cfg(test)]
mod rung8_tests;
#[cfg(test)]
mod rung9_tests;
pub mod type_aliases;
#[cfg(test)]
mod upstream_734_gc_tests;

// C++ macros are global #defines; translated callers use them unqualified.
// Pull every #[macro_export] macro from luau-common into textual scope so
// LUAU_INSN_OP!/LUAU_ASSERT!/... resolve without per-file imports.
#[macro_use]
extern crate luaur_common;
