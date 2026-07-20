use crate::macros::f_is_lua::f_isLua;
use crate::macros::ttisfunction::ttisfunction;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! isLua {
    ($ci:expr) => {
        $crate::macros::ttisfunction::ttisfunction!((*$ci).func)
            && $crate::macros::f_is_lua::f_isLua!($ci)
    };
}

pub use isLua;
