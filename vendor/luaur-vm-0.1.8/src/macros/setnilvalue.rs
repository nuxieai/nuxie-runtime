use crate::enums::lua_type::lua_Type;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setnilvalue {
    ($obj:expr) => {
        (*$obj).tt = $crate::enums::lua_type::lua_Type::LUA_TNIL as i32;
    };
}

pub use setnilvalue;
