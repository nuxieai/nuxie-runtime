use crate::enums::lua_type::lua_Type;
use crate::functions::lua_type::lua_type;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_isbuffer {
    ($L:expr, $n:expr) => {
        unsafe { $crate::functions::lua_type::lua_type($L, $n) }
            == $crate::enums::lua_type::lua_Type::LUA_TBUFFER as i32
    };
}

pub use lua_isbuffer;
