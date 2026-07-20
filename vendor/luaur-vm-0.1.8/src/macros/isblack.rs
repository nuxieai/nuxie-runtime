use crate::macros::testbit::testbit;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! isblack {
    ($x:expr) => {
        $crate::macros::testbit::testbit!(
            (*$x).gch.marked as i32,
            $crate::macros::maskmarks::BLACKBIT
        ) != 0
    };
}

pub use isblack;
