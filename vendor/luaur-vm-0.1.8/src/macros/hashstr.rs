#[allow(non_snake_case)]
#[macro_export]
macro_rules! hashstr {
    ($t:expr, $str:expr) => {
        $crate::macros::hashpow_2::hashpow2!($t, unsafe { (*$str).hash })
    };
}

pub use hashstr;
