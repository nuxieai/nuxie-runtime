//! Source: `VM/src/lobject.h:73` (hand-ported)
// #define pvalue(o) check_exp(ttislightuserdata(o), (o)->value.p)
#[allow(non_snake_case)]
#[macro_export]
macro_rules! pvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttislightuserdata::ttislightuserdata!($o),
            (*$o).value.p
        )
    };
}
pub use pvalue;
