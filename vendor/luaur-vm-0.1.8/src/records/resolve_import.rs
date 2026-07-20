use crate::records::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ResolveImport {
    pub k: *mut TValue,
    pub id: u32,
}
