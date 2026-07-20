#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LuauObject {
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) memcat: u8,

    pub gclist: *mut crate::records::gc_object::GcObject,

    /// The class object that this value is an instance of.
    pub lclass: *mut crate::records::luau_class::LuauClass,

    /// The number of members that this instance contains. We need this in order
    /// to free ourselves if we got swept in the same GC cycle as our class
    /// pointer.
    pub numberofmembers: core::ffi::c_int,

    /// The fields of this instance.
    pub members: *mut crate::records::lua_t_value::TValue,
}

impl LuauObject {
    pub const CommonHeader: () = ();
}
