use crate::macros::reset_2_bits::reset2bits;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! white2gray {
    ($x:expr) => {
        $crate::macros::reset_2_bits::reset2bits!(
            (*$x).gch.marked,
            $crate::macros::fixedbit::WHITE0BIT,
            $crate::macros::fixedbit::WHITE1BIT
        )
    };
}

pub use white2gray;
