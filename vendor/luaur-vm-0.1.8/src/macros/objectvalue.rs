use crate::macros::check_exp::check_exp;
use crate::macros::ttisobject::ttisobject;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! objectvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisobject::ttisobject!($o),
            &mut (*(*$o).value.gc).lobject
        )
    };
}

pub use objectvalue;
