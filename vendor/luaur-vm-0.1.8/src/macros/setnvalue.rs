use crate::enums::lua_type::lua_Type;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setnvalue {
    ($obj:expr, $x:expr) => {
        unsafe {
            let i_o = $obj;
            (*i_o).value.n = $x;
            (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TNUMBER as i32;
        }
    };
}

pub use setnvalue;
