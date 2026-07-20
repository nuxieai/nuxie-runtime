use crate::enums::lua_type::lua_Type;
use crate::macros::check_exp::check_exp;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gco2h {
    ($o:expr) => {{
        unsafe {
            $crate::macros::check_exp::check_exp!(
                (*$o).gch.tt == ($crate::enums::lua_type::lua_Type::LUA_TTABLE as u8),
                core::ptr::addr_of_mut!((*$o).h) as *mut $crate::records::lua_table::LuaTable
            )
        }
    }};
}

pub use gco2h;
