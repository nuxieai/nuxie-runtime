use crate::macros::bvalue::bvalue;
use crate::macros::ttisboolean::ttisboolean;
use crate::macros::ttisnil::ttisnil;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! l_isfalse {
    ($o:expr) => {
        $crate::macros::ttisnil::ttisnil!($o)
            || ($crate::macros::ttisboolean::ttisboolean!($o)
                && $crate::macros::bvalue::bvalue!($o) == 0)
    };
}

pub use l_isfalse;
