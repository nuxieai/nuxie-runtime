#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CallContext {
    pub(crate) t: *mut crate::records::lua_table::LuaTable,
    pub(crate) nhsize: core::ffi::c_int,
}
