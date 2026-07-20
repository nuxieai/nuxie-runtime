//! `LUAU_FASTINT(flag)` — forward-declares an int FastFlag defined in another
//! translation unit. Reference: `luau/Common/include/Luau/Common.h`. Expands to
//! nothing (the flag is a `pub static` reached by path); see
//! [`crate::macros::luau_fastflag`].

#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_FASTINT {
    ($flag:ident) => {};
}

pub use LUAU_FASTINT;
