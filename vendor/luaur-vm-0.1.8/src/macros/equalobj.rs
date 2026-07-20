use crate::functions::lua_v_equalval::lua_v_equalval;
use crate::macros::ttype::ttype;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! equalobj {
    ($L:expr, $o1:expr, $o2:expr) => {
        ($crate::macros::ttype::ttype!($o1) == $crate::macros::ttype::ttype!($o2)
            && $crate::functions::lua_v_equalval::lua_v_equalval($L, $o1, $o2) != 0)
    };
}

pub use equalobj;
