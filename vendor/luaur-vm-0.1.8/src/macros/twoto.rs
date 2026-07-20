#[allow(non_snake_case)]
#[macro_export]
macro_rules! twoto {
    ($x:expr) => {
        (1 << ($x)) as core::ffi::c_int
    };
}

pub use twoto;
