use crate::macros::bitmask::bitmask;
use crate::macros::resetbits::resetbits;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! resetbit {
    ($x:expr, $b:expr) => {
        $crate::macros::resetbits::resetbits!($x, $crate::macros::bitmask::bitmask($b))
    };
}

pub use resetbit;
