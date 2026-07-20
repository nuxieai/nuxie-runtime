#[allow(non_snake_case)]
#[macro_export]
macro_rules! cast_to {
    ($t:ty, $exp:expr) => {
        $exp as $t
    };
}

pub use cast_to;
