use crate::enums::lua_type::lua_Type;
use crate::functions::lua_type::lua_type;

#[macro_export]
macro_rules! lua_istable {
    ($l:expr, $n:expr) => {
        unsafe {
            $crate::functions::lua_type::lua_type($l, $n)
                == ($crate::enums::lua_type::lua_Type::LUA_TTABLE as i32)
        }
    };
}

pub use lua_istable;
