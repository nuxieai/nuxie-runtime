use crate::type_aliases::lua_compile_constant::lua_CompileConstant;

#[allow(non_camel_case_types)]
pub type lua_LibraryMemberConstantCallback = Option<
    unsafe extern "C" fn(
        library: *const core::ffi::c_char,
        member: *const core::ffi::c_char,
        constant: *mut lua_CompileConstant,
    ),
>;
