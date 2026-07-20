use crate::macros::check_exp::check_exp;
use crate::macros::ttisfunction::ttisfunction;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! clvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisfunction::ttisfunction!($o),
            core::ptr::addr_of_mut!((*(*$o).value.gc).cl) as *mut $crate::records::closure::Closure
        )
    };
}

pub use clvalue;
