#[allow(non_snake_case)]
#[macro_export]
macro_rules! changewhite {
    ($x:expr) => {
        (*$x).gch.marked ^= $crate::macros::whitebits::WHITEBITS
    };
}

pub use changewhite;
