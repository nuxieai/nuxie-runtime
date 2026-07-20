use crate::enums::lua_type::lua_Type;
use crate::macros::check_exp::check_exp;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gco2cl {
    ($o:expr) => {{
        unsafe {
            $crate::macros::check_exp::check_exp!(
                (*$o).gch.tt == ($crate::enums::lua_type::lua_Type::LUA_TFUNCTION as u8),
                core::ptr::addr_of_mut!((*$o).cl) as *mut $crate::records::closure::Closure
            )
        }
    }};
}

pub use gco2cl;
