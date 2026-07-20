#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_LIKELY {
    ($x:expr) => {
        $x
    };
}

pub use LUAU_LIKELY;
