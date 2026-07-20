use crate::macros::test_2_bits::test2bits;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! iswhite {
    ($x:expr) => {
        ($crate::macros::test_2_bits::test2bits!(
            (*$x).gch.marked as i32,
            $crate::macros::fixedbit::WHITE0BIT,
            $crate::macros::fixedbit::WHITE1BIT
        ) != 0)
    };
}

pub use iswhite;
