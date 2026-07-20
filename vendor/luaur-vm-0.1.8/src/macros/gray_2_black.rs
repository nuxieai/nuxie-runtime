use crate::macros::l_setbit::l_setbit;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gray2black {
    ($x:expr) => {
        $crate::macros::l_setbit::l_setbit!((*$x).gch.marked, $crate::macros::maskmarks::BLACKBIT)
    };
}

pub use gray2black;
