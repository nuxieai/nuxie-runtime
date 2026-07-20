//! `LUAU_FASTFLAG(flag)` — forward-declares a bool FastFlag defined in another
//! translation unit. Reference: `luau/Common/include/Luau/Common.h`
//! (`namespace FFlag { extern FValue<bool> flag; }`).
//!
//! Rust has no cross-module `extern` declaration for statics: the flag is a
//! `pub static` reachable by path (`crate::FFlag::flag`), so this expands to
//! nothing. The accompanying `LUAU_FASTFLAGVARIABLE` is what defines it.

#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_FASTFLAG {
    ($flag:ident) => {};
}

pub use LUAU_FASTFLAG;
