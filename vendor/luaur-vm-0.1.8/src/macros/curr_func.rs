use crate::macros::clvalue::clvalue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! curr_func {
    ($L:expr) => {
        $crate::macros::clvalue::clvalue!((*(*$L).ci).func)
    };
}

pub use curr_func;
