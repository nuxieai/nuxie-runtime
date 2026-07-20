use crate::enums::lua_type::lua_Type;
use crate::macros::check_exp::check_exp;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gco2th {
    ($o:expr) => {{
        unsafe {
            $crate::macros::check_exp::check_exp!(
                (*$o).gch.tt == ($crate::enums::lua_type::lua_Type::LUA_TTHREAD as u8),
                core::ptr::addr_of_mut!((*$o).th) as *mut $crate::records::lua_state::lua_State
            )
        }
    }};
}

pub use gco2th;
