use crate::macros::cast_to::cast_to;
use crate::macros::whitebits::WHITEBITS;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaC_white {
    ($g:expr) => {
        $crate::macros::cast_to::cast_to!(
            u8,
            (unsafe { (*$g).currentwhite } as i32) & $crate::macros::whitebits::WHITEBITS
        )
    };
}

pub use luaC_white;
