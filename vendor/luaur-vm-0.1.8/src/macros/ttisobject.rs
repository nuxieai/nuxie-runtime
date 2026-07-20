use crate::enums::lua_type::lua_Type;
use crate::macros::ttype::ttype;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! ttisobject {
    ($o:expr) => {
        $crate::macros::ttype::ttype!($o) == ($crate::enums::lua_type::lua_Type::LUA_TOBJECT as i32)
    };
}

pub use ttisobject;
