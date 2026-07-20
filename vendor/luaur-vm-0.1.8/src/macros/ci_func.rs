use crate::macros::clvalue::clvalue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! ci_func {
    ($ci:expr) => {
        $crate::macros::clvalue::clvalue!((*$ci).func)
    };
}

pub use ci_func;
