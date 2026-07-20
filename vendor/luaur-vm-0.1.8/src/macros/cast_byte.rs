#[allow(non_snake_case)]
#[macro_export]
macro_rules! cast_byte {
    ($i:expr) => {
        $crate::macros::cast_to::cast_to!(u8, $i)
    };
}

pub use cast_byte;
