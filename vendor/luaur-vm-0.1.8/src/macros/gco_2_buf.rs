use crate::enums::lua_type::lua_Type;
use crate::macros::check_exp::check_exp;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gco2buf {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            (*$o).gch.tt == ($crate::enums::lua_type::lua_Type::LUA_TBUFFER as u8),
            &(*$o).buf
        )
    };
}

pub use gco2buf;
