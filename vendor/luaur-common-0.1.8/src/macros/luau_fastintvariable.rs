//! `LUAU_FASTINTVARIABLE(flag, def)` — defines a static (non-dynamic) int
//! FastFlag. Reference: `luau/Common/include/Luau/Common.h`. See
//! [`crate::macros::luau_fastflagvariable`] for the namespace/`pub static` design.

#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_FASTINTVARIABLE {
    ($flag:ident, $def:expr) => {
        #[allow(non_upper_case_globals)]
        pub static $flag: $crate::records::f_value::FValue<i32> =
            $crate::records::f_value::FValue::new(
                concat!(stringify!($flag), "\0").as_ptr() as *const core::ffi::c_char,
                $def,
                false,
            );
    };
}

pub use LUAU_FASTINTVARIABLE;
