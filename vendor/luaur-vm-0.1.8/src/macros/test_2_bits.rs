use crate::macros::bit_2_mask::bit2mask;
use crate::macros::testbits::testbits;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! test2bits {
    ($x:expr, $b1:expr, $b2:expr) => {
        $crate::macros::testbits::testbits($x, $crate::macros::bit_2_mask::bit2mask($b1, $b2))
    };
}

pub use test2bits;
