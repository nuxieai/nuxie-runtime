use crate::macros::getstr::getstr;
use crate::macros::tsvalue::tsvalue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! svalue {
    ($o:expr) => {
        unsafe { $crate::macros::getstr::getstr($crate::macros::tsvalue::tsvalue!($o)) }
    };
}

pub use svalue;
