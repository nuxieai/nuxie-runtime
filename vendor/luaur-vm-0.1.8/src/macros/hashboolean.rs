#[allow(non_snake_case)]
#[macro_export]
macro_rules! hashboolean {
    ($t:expr, $p:expr) => {
        $crate::macros::hashpow_2::hashpow2!($t, $p)
    };
}

pub use hashboolean;
