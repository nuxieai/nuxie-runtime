use crate::macros::checkliveness::checkliveness;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setobj {
    ($L:expr, $obj1:expr, $obj2:expr) => {
        unsafe {
            let o2: *const $crate::type_aliases::t_value::TValue = $obj2;
            let o1: *mut $crate::type_aliases::t_value::TValue = $obj1;
            *o1 = *o2;
            $crate::macros::checkliveness::checkliveness!((*$L).global, o1);
        }
    };
}

pub use setobj;
