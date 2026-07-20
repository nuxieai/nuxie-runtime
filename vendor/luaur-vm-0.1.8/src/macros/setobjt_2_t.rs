use crate::macros::setobj::setobj;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setobjt_2_t {
    ($L:expr, $obj1:expr, $obj2:expr) => {
        $crate::macros::setobj::setobj!($L, $obj1, $obj2)
    };
}

pub use setobjt_2_t;

// C name
#[allow(unused_imports)]
pub use setobjt_2_t as setobjt2t;
