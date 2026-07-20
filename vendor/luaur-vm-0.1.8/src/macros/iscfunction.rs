use crate::enums::lua_type::lua_Type;
use crate::macros::clvalue::clvalue;
use crate::macros::ttype::ttype;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! iscfunction {
    ($o:expr) => {
        $crate::macros::ttype::ttype!($o)
            == ($crate::enums::lua_type::lua_Type::LUA_TFUNCTION as i32)
            && unsafe { (*$crate::macros::clvalue::clvalue!($o)).isC != 0 }
    };
}

pub use iscfunction;
