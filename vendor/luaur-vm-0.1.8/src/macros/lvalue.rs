use crate::macros::check_exp::check_exp;
use crate::macros::ttisinteger::ttisinteger;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisinteger::ttisinteger!($o),
            (*$o).value.l
        )
    };
}

pub use lvalue;
