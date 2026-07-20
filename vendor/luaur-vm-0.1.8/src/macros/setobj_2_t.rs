use crate::macros::setobj::setobj;

#[allow(non_upper_case_globals)]
#[macro_export]
macro_rules! setobj2t {
    ($L:expr, $obj1:expr, $obj2:expr) => {
        $crate::macros::setobj::setobj!($L, $obj1, $obj2);
    };
}

pub use setobj2t;
