use crate::records::lua_t_value::lua_TValue;
use crate::type_aliases::lua_userdata_direct_access::lua_UserdataDirectAccess;
use crate::type_aliases::lua_userdata_direct_namecall::lua_UserdataDirectNamecall;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct lua_UdataDirectAccessData {
    pub(crate) indextm: lua_TValue,
    pub(crate) newindextm: lua_TValue,
    pub(crate) namecalltm: lua_TValue,
    pub(crate) index: lua_UserdataDirectAccess,
    pub(crate) newindex: lua_UserdataDirectAccess,
    pub(crate) namecall: lua_UserdataDirectNamecall,
}

impl Default for lua_UdataDirectAccessData {
    fn default() -> Self {
        Self {
            indextm: lua_TValue::default(),
            newindextm: lua_TValue::default(),
            namecalltm: lua_TValue::default(),
            index: None,
            newindex: None,
            namecall: None,
        }
    }
}

#[allow(non_camel_case_types)]
pub type LuaUdataDirectAccessData = lua_UdataDirectAccessData;
