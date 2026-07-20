use crate::macros::check_exp::check_exp;
use crate::macros::ttisbuffer::ttisbuffer;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! bufvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisbuffer::ttisbuffer!($o),
            &mut (*(*$o).value.gc).buf
        )
    };
}

pub use bufvalue;
