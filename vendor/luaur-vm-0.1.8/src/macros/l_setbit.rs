use crate::macros::bitmask::bitmask;
use crate::macros::setbits::setbits;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! l_setbit {
    ($x:expr, $b:expr) => {
        $crate::macros::setbits::setbits!($x, $crate::macros::bitmask::bitmask($b))
    };
}

pub use l_setbit;
