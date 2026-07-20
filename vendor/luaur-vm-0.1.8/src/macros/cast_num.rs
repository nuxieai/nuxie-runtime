use crate::macros::cast_to::cast_to;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! cast_num {
    ($i:expr) => {
        $crate::macros::cast_to::cast_to!(f64, ($i))
    };
}

pub use cast_num;
