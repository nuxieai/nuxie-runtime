use crate::macros::ttype::ttype;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setttype {
    ($obj:expr, $tt:expr) => {
        (*$obj).set_tt($tt);
    };
}

pub use setttype;
