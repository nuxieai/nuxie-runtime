use crate::macros::bit_2_mask::bit2mask;
use crate::macros::setbits::setbits;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! set2bits {
    ($x:expr, $b1:expr, $b2:expr) => {
        $crate::macros::setbits::setbits!($x, $crate::macros::bit_2_mask::bit2mask($b1, $b2))
    };
}

pub use set2bits;
