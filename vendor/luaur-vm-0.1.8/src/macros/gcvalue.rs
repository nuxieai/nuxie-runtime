use crate::macros::check_exp::check_exp;
use crate::macros::iscollectable::iscollectable;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gcvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::iscollectable::iscollectable!($o),
            (*$o).value.gc
        )
    };
}

pub use gcvalue;
