use crate::enums::lua_type::lua_Type;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setlvalue {
    ($obj:expr, $x:expr) => {
        unsafe {
            let i_o = $obj;
            (*i_o).value.l = $x;
            (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TINTEGER as i32;
        }
    };
}

pub use setlvalue;
