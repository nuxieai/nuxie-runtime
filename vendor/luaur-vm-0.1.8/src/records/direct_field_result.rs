use crate::records::lua_state::lua_State;
use crate::records::lua_t_value::TValue;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirectFieldResult {
    pub l: *mut lua_State,
    pub slot: *mut TValue,
}
