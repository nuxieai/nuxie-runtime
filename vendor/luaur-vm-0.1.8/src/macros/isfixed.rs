use crate::macros::testbit::testbit;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! isfixed {
    ($x:expr) => {
        $crate::macros::testbit::testbit!((*$x).gch.marked, $crate::macros::FIXEDBIT)
    };
}

pub use isfixed;
