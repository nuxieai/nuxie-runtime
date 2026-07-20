#[allow(non_snake_case)]
#[macro_export]
macro_rules! resetbits {
    ($x:expr, $m:expr) => {
        $x &= $crate::macros::cast_to::cast_to!(u8, !($m))
    };
}

pub use resetbits;
