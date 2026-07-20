use crate::records::lua_table::LuaTable;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gnode {
    ($t:expr, $i:expr) => {
        unsafe { (*$t).node.add($i as usize) }
    };
}

pub use gnode;
