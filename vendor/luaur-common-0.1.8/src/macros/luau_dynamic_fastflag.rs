//! `LUAU_DYNAMIC_FASTFLAG(flag)` — forward-declares a dynamic bool FastFlag
//! defined in another translation unit. Reference:
//! `luau/Common/include/Luau/Common.h`. Expands to nothing (the flag is a
//! `pub static` reached by path, `crate::DFFlag::flag`); see
//! [`crate::macros::luau_fastflag`].

#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_DYNAMIC_FASTFLAG {
    ($flag:ident) => {};
}

pub use LUAU_DYNAMIC_FASTFLAG;
