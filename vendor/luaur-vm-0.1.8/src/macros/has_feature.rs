#[allow(non_snake_case)]
#[macro_export]
macro_rules! __has_feature {
    ($x:ident) => {
        0
    };
}

pub use __has_feature;
