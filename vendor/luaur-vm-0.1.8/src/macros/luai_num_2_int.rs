#[allow(non_snake_case)]
#[macro_export]
macro_rules! luai_num2int {
    ($i:expr, $d:expr) => {
        $i = $d as core::ffi::c_int
    };
}

pub use luai_num2int;
