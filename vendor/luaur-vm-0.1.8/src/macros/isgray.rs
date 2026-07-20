#[allow(non_snake_case)]
#[macro_export]
macro_rules! isgray {
    ($x:expr) => {
        $crate::macros::testbits::testbits(
            (*$x).gch.marked as i32,
            $crate::macros::whitebits::WHITEBITS
                | $crate::macros::bitmask::bitmask($crate::macros::blackbit::BLACKBIT as i32),
        ) == 0
    };
}

pub use isgray;
