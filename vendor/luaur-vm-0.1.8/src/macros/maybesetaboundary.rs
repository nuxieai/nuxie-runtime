#[allow(non_snake_case)]
#[macro_export]
macro_rules! maybesetaboundary {
    ($t:expr, $boundary:expr) => {
        if (*$t).aboundary <= 0 {
            (*$t).aboundary = -($boundary as core::ffi::c_int);
        }
    };
}

pub use maybesetaboundary;
