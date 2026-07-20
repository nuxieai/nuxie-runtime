#[allow(non_snake_case)]
#[macro_export]
macro_rules! hashpow2 {
    ($t:expr, $n:expr) => {
        $crate::macros::gnode::gnode!(
            $t,
            $crate::macros::lmod::lmod!($n, $crate::macros::sizenode::sizenode!($t)) as usize
        )
    };
}

pub use hashpow2;
