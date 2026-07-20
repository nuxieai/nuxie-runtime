#[allow(non_snake_case)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union LuaTable_Union {
    pub lastfree: core::ffi::c_int,
    pub aboundary: core::ffi::c_int,
}

impl core::fmt::Debug for LuaTable_Union {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LuaTable_Union").finish_non_exhaustive()
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LuaTable {
    pub tt: u8,
    pub marked: u8,
    pub memcat: u8,

    pub tmcache: u8,
    pub readonly: u8,
    pub safeenv: u8,
    pub lsizenode: u8,
    pub nodemask8: u8,

    pub sizearray: core::ffi::c_int,
    pub union: LuaTable_Union,

    pub metatable: *mut crate::records::lua_table::LuaTable,
    pub array: *mut crate::records::lua_t_value::TValue,
    pub node: *mut crate::records::lua_node::LuaNode,
    pub gclist: *mut crate::records::gc_object::GcObject,
}

#[allow(non_upper_case_globals)]
impl LuaTable {
    pub const lastfree: () = ();
    pub const aboundary: () = ();
}

#[allow(non_camel_case_types)]
pub type lua_table = LuaTable;
