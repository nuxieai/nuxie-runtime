use crate::macros::check_exp::check_exp;
use crate::macros::ttisstring::ttisstring;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! tsvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisstring::ttisstring!($o),
            core::ptr::addr_of!((*(*$o).value.gc).ts) as *const _
                as *const $crate::records::t_string::TString
        )
    };
}

pub use tsvalue;
