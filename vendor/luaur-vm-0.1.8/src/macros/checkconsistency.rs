use crate::macros::iscollectable::iscollectable;
use crate::macros::ttype::ttype;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! checkconsistency {
    ($obj:expr) => {
        $crate::macros::LUAU_ASSERT::LUAU_ASSERT!(
            !$crate::macros::iscollectable::iscollectable!($obj)
                || ($crate::macros::ttype::ttype!($obj) == (*$obj).value.gc.gch.tt)
        )
    };
}

pub use checkconsistency;
