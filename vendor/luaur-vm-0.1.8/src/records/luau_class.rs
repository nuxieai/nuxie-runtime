#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LuauClass {
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) memcat: u8,

    pub gclist: *mut crate::records::gc_object::GcObject,

    pub name: *mut crate::records::t_string::TString,

    pub staticmembers: *mut crate::records::lua_t_value::TValue,

    pub memberstooffset: *mut crate::records::lua_table::LuaTable,

    pub offsettomember: *mut *mut crate::records::t_string::TString,

    pub metatable: *mut crate::records::lua_table::LuaTable,

    pub instancemetatable: *mut crate::records::lua_table::LuaTable,

    pub numberofinstancemembers: core::ffi::c_int,

    pub numberofallmembers: core::ffi::c_int,
}
