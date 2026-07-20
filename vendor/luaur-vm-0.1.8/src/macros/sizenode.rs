#[allow(non_snake_case)]
#[macro_export]
macro_rules! sizenode {
    ($t:expr) => {
        $crate::macros::twoto::twoto!(unsafe { (*$t).lsizenode })
    };
}

pub use sizenode;
