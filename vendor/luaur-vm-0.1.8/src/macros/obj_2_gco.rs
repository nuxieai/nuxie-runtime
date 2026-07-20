use crate::macros::cast_to::cast_to;
use crate::macros::check_exp::check_exp;
use crate::macros::iscollectable::iscollectable;
use crate::records::gc_object::GCObject;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! obj2gco {
    ($v:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::iscollectable::iscollectable!($v),
            $crate::macros::cast_to::cast_to!(*mut $crate::records::gc_object::GCObject, $v)
        )
    };
}

pub use obj2gco;
