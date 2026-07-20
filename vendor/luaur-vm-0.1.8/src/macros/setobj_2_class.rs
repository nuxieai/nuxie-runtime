use crate::macros::setobj::setobj;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setobj2class {
    ($L:expr, $obj1:expr, $obj2:expr) => {
        $crate::macros::setobj::setobj!($L, $obj1, $obj2);
    };
}

pub use setobj2class;
