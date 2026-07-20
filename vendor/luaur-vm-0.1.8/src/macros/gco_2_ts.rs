use crate::enums::lua_type::lua_Type;
use crate::macros::check_exp::check_exp;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gco2ts {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            (*$o).gch.tt == ($crate::enums::lua_type::lua_Type::LUA_TSTRING as u8),
            &(*$o).ts
        )
    };
}

pub use gco2ts;
