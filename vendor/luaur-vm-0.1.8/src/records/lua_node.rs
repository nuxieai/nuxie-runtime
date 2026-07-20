#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LuaNode {
    pub val: crate::records::lua_t_value::TValue,
    pub key: crate::records::t_key::TKey,
}

impl Default for LuaNode {
    fn default() -> Self {
        Self {
            val: crate::records::lua_t_value::TValue::default(),
            key: crate::records::t_key::TKey::default(),
        }
    }
}

#[allow(non_camel_case_types)]
pub type lua_node = LuaNode;
