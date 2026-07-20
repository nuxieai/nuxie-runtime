use crate::macros::check_exp::check_exp;
use crate::macros::ttisclass::ttisclass;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! classvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisclass::ttisclass!($o),
            &mut (*(*$o).value.gc).lclass
        )
    };
}

pub use classvalue;
