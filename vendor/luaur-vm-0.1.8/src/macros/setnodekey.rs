use crate::macros::checkliveness::checkliveness;
use crate::records::lua_node::LuaNode;
use crate::records::lua_t_value::lua_TValue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setnodekey {
    ($L:expr, $node:expr, $obj:expr) => {
        unsafe {
            let n_: *mut LuaNode = $node as *mut LuaNode;
            let i_o: *const lua_TValue = $obj as *const lua_TValue;

            (*n_).key.value = (*i_o).value;
            core::ptr::copy_nonoverlapping((*i_o).extra.as_ptr(), (*n_).key.extra.as_mut_ptr(), 1);
            (*n_).key.set_tt((*i_o).tt);
            checkliveness!((*$L).global, i_o as *mut TValue);
        }
    };
}

pub use setnodekey;
