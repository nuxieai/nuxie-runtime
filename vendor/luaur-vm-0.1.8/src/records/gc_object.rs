#[allow(non_camel_case_types)]
#[repr(C)]
pub union GcObject {
    pub gch: crate::records::g_cheader::GCheader,
    pub ts: core::mem::ManuallyDrop<crate::records::t_string::TString>,
    pub u: core::mem::ManuallyDrop<crate::records::udata::Udata>,
    pub cl: core::mem::ManuallyDrop<crate::records::closure::Closure>,
    pub h: core::mem::ManuallyDrop<crate::records::lua_table::LuaTable>,
    pub p: core::mem::ManuallyDrop<crate::records::proto::Proto>,
    pub uv: core::mem::ManuallyDrop<crate::records::up_val::UpVal>,
    pub th: core::mem::ManuallyDrop<crate::records::lua_state::lua_State>,
    pub buf: core::mem::ManuallyDrop<crate::records::luau_buffer::LuauBuffer>,
    pub lclass: core::mem::ManuallyDrop<crate::records::luau_class::LuauClass>,
    pub lobject: core::mem::ManuallyDrop<crate::records::luau_object::LuauObject>,
}

#[allow(non_camel_case_types)]
pub type GCObject = GcObject;
