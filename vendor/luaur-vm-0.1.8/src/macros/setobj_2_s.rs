use crate::macros::setobj::setobj;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setobj_2_s {
    ($L:expr, $obj1:expr, $obj2:expr) => {
        $crate::macros::setobj::setobj!($L, $obj1, $obj2)
    };
}

pub use setobj_2_s;

// C name
#[allow(unused_imports)]
pub use setobj_2_s as setobj2s;
