use crate::macros::bitmask::bitmask;
use crate::macros::testbits::testbits;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! testbit {
    ($x:expr, $b:expr) => {
        $crate::macros::testbits::testbits($x, $crate::macros::bitmask::bitmask($b))
    };
}

pub use testbit;
