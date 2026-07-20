use crate::enums::lua_type::lua_Type;
use crate::functions::lua_v_tostring::lua_v_tostring;
use crate::macros::ttype::ttype;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! tostring {
    ($L:expr, $o:expr) => {
        ($crate::macros::ttype::ttype!($o)
            == ($crate::enums::lua_type::lua_Type::LUA_TSTRING as i32))
            || ($crate::functions::lua_v_tostring::lua_v_tostring($L, $o) != 0)
    };
}

pub use tostring;
