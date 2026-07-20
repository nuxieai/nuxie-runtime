use crate::enums::lua_type::lua_Type;
use crate::functions::lua_type::lua_type;

#[macro_export]
macro_rules! lua_islightuserdata {
    ($l:expr, $n:expr) => {
        unsafe {
            $crate::functions::lua_type::lua_type($l, $n)
                == ($crate::enums::lua_type::lua_Type::LUA_TLIGHTUSERDATA as i32)
        }
    };
}

pub use lua_islightuserdata;
