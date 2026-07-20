use crate::macros::bit_2_mask::bit2mask;
use crate::macros::resetbits::resetbits;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! reset2bits {
    ($x:expr, $b1:expr, $b2:expr) => {
        $crate::macros::resetbits::resetbits!($x, $crate::macros::bit_2_mask::bit2mask($b1, $b2))
    };
}

pub use reset2bits;
