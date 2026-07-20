#[allow(non_snake_case)]
#[macro_export]
macro_rules! nvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisnumber::ttisnumber!($o),
            (*$o).value.n
        )
    };
}

pub use nvalue;
