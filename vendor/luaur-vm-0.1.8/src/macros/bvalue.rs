use crate::macros::check_exp::check_exp;
use crate::macros::ttisboolean::ttisboolean;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! bvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisboolean::ttisboolean!($o),
            (*$o).value.b
        )
    };
}

pub use bvalue;
