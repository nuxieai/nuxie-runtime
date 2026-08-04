use crate::type_aliases::library_member_constant_callback::LibraryMemberConstantCallback;
use crate::type_aliases::library_member_type_callback::LibraryMemberTypeCallback;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    pub optimization_level: core::ffi::c_int,
    pub debug_level: core::ffi::c_int,
    pub type_info_level: core::ffi::c_int,
    pub coverage_level: core::ffi::c_int,
    pub vector_lib: *const core::ffi::c_char,
    pub vector_ctor: *const core::ffi::c_char,
    pub vector_type: *const core::ffi::c_char,
    pub mutable_globals: *const *const core::ffi::c_char,
    pub userdata_types: *const *const core::ffi::c_char,
    pub libraries_with_known_members: *const *const core::ffi::c_char,
    pub library_member_type_cb: LibraryMemberTypeCallback,
    pub library_member_constant_cb: LibraryMemberConstantCallback,
    pub disabled_builtins: *const *const core::ffi::c_char,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            optimization_level: 1,
            debug_level: 1,
            type_info_level: 0,
            coverage_level: 0,
            vector_lib: core::ptr::null(),
            vector_ctor: core::ptr::null(),
            vector_type: core::ptr::null(),
            mutable_globals: core::ptr::null(),
            userdata_types: core::ptr::null(),
            libraries_with_known_members: core::ptr::null(),
            library_member_type_cb: None,
            library_member_constant_cb: None,
            disabled_builtins: core::ptr::null(),
        }
    }
}
