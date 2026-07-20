use crate::macros::ci_func::ci_func;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! f_isLua {
    ($ci:expr) => {
        // C: `#define f_isLua(ci) (!ci_func(ci)->isC)` — logical NOT, i.e. isC == 0.
        (*$crate::macros::ci_func::ci_func!($ci)).isC == 0
    };
}

pub use f_isLua;
