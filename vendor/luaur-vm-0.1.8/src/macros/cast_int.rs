use crate::macros::cast_to::cast_to;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! cast_int {
    ($i:expr) => {
        $crate::macros::cast_to::cast_to!(core::ffi::c_int, $i)
    };
}

pub use cast_int;
