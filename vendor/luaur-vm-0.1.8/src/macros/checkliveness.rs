use crate::iscollectable;
use crate::isdead;
use crate::ttype;
use luaur_common::LUAU_ASSERT;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! checkliveness {
    ($g:expr, $obj:expr) => {
        luaur_common::LUAU_ASSERT!(
            !$crate::iscollectable!($obj)
                || (($crate::ttype!($obj) == (*(*$obj).value.gc).gch.tt as core::ffi::c_int)
                    && !$crate::isdead!($g, (*$obj).value.gc))
        )
    };
}

pub use checkliveness;
