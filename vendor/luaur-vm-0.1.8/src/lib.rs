extern crate alloc;

pub mod enums;
pub mod functions;
pub mod macros;
pub mod methods;
pub mod records;
pub mod type_aliases;

// C++ macros are global #defines; translated callers use them unqualified.
// Pull every #[macro_export] macro from luau-common into textual scope so
// LUAU_INSN_OP!/LUAU_ASSERT!/... resolve without per-file imports.
#[macro_use]
extern crate luaur_common;
