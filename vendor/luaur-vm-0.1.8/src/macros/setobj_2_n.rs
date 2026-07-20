use crate::macros::setobj::setobj;

#[allow(non_upper_case_globals)]
#[macro_export]
macro_rules! SETOBJ_2_N {
    ($L:expr, $obj1:expr, $obj2:expr) => {
        $crate::macros::setobj::setobj!($L, $obj1, $obj2);
    };
}

pub use SETOBJ_2_N;

// C name
#[allow(unused_imports)]
pub use SETOBJ_2_N as setobj2n;
