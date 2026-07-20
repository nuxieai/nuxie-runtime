//! `LUAU_FASTFLAGVARIABLE(flag)` — defines a static (non-dynamic) bool FastFlag.
//! Reference: `luau/Common/include/Luau/Common.h`.
//!
//! C++ expands to `namespace FFlag { FValue<bool> flag(#flag, false, false); }`.
//! Rust modules aren't open like C++ namespaces, so the macro emits a bare
//! `pub static` at the call site (no per-flag `mod`, which would collide when a
//! file defines two flags); the enclosing per-crate `FFlag` module supplies the
//! namespace, so reads stay `FFlag::flag` -> `crate::FFlag::flag.get()`.

#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_FASTFLAGVARIABLE {
    ($flag:ident) => {
        #[allow(non_upper_case_globals)]
        pub static $flag: $crate::records::f_value::FValue<bool> =
            $crate::records::f_value::FValue::new(
                concat!(stringify!($flag), "\0").as_ptr() as *const core::ffi::c_char,
                false,
                false,
            );
    };
}

pub use LUAU_FASTFLAGVARIABLE;
