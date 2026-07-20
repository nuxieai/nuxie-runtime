use crate::enums::lua_type::lua_Type;
use crate::macros::ttype::ttype;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! ttisnil {
    ($o:expr) => {
        $crate::macros::ttype::ttype!($o) == ($crate::enums::lua_type::lua_Type::LUA_TNIL as i32)
    };
}

pub use ttisnil;
