use crate::macros::fixedbit::FIXEDBIT;
use crate::macros::l_setbit::l_setbit;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaS_fix {
    ($s:expr) => {
        $crate::macros::l_setbit::l_setbit!((*$s).hdr.marked, $crate::macros::fixedbit::FIXEDBIT)
    };
}

pub use luaS_fix;
