//! `LUAU_DYNAMIC_FASTINTVARIABLE(flag, def)` — defines a *dynamic* int FastFlag
//! (the `dynamic` bit is `true`). Reference: `luau/Common/include/Luau/Common.h`.
//! See [`crate::macros::luau_fastflagvariable`] for the namespace/`pub static`
//! design; reads are `DFInt::flag` -> `crate::DFInt::flag.get()`.

#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_DYNAMIC_FASTINTVARIABLE {
    ($flag:ident, $def:expr) => {
        #[allow(non_upper_case_globals)]
        pub static $flag: $crate::records::f_value::FValue<i32> =
            $crate::records::f_value::FValue::new(
                concat!(stringify!($flag), "\0").as_ptr() as *const core::ffi::c_char,
                $def,
                true,
            );
    };
}

pub use LUAU_DYNAMIC_FASTINTVARIABLE;
